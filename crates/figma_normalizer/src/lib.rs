#![forbid(unsafe_code)]

use serde_json::Value;

pub const NORMALIZED_SCHEMA_VERSION: &str = "1.0";
pub const FIGMA_API_VERSION: &str = "v1";

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum NormalizationError {
    #[error("missing required payload field: {0}")]
    MissingRequiredPayloadField(String),
    #[error("invalid payload field: {0}")]
    InvalidPayloadField(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NormalizationWarning {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizationOutput {
    pub document: NormalizedDocument,
    pub warnings: Vec<NormalizationWarning>,
}

pub fn normalize_snapshot(
    snapshot: &figma_client::RawFigmaSnapshot,
) -> Result<NormalizationOutput, NormalizationError> {
    let payload = snapshot.payload.as_object().ok_or_else(|| {
        NormalizationError::InvalidPayloadField("payload must be a JSON object".to_string())
    })?;
    let root = payload
        .get("document")
        .ok_or_else(|| NormalizationError::MissingRequiredPayloadField("document".to_string()))?;

    let mut nodes = Vec::new();
    let mut warnings = Vec::new();
    let root_node_id = normalize_node(root, None, &mut nodes, &mut warnings)?;

    let document = NormalizedDocument {
        schema_version: NORMALIZED_SCHEMA_VERSION.to_string(),
        source: NormalizedSource {
            file_key: snapshot.source.file_key.clone(),
            root_node_id,
            figma_api_version: snapshot.source.figma_api_version.clone(),
        },
        nodes,
    };

    Ok(NormalizationOutput { document, warnings })
}

fn normalize_node(
    node_value: &Value,
    parent_id: Option<&str>,
    nodes: &mut Vec<NormalizedNode>,
    warnings: &mut Vec<NormalizationWarning>,
) -> Result<String, NormalizationError> {
    let node = node_value.as_object().ok_or_else(|| {
        NormalizationError::InvalidPayloadField("node must be a JSON object".to_string())
    })?;

    let id = required_string(node, "id")?;
    let name = node
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let kind = map_node_kind(
        node.get("type").and_then(Value::as_str),
        id.as_str(),
        warnings,
    );
    let visible = node.get("visible").and_then(Value::as_bool).unwrap_or(true);
    let bounds = parse_bounds(node.get("absoluteBoundingBox"))?;

    append_unsupported_field_warnings(node, id.as_str(), warnings);

    let node_index = nodes.len();
    nodes.push(NormalizedNode {
        id: id.clone(),
        parent_id: parent_id.map(str::to_string),
        name,
        kind,
        visible,
        bounds,
        layout: None,
        constraints: None,
        style: default_style(),
        component: default_component(),
        children: Vec::new(),
    });

    let children = parse_children(node.get("children"))?;
    let mut child_ids = Vec::new();
    for child in children {
        let child_id = normalize_node(child, Some(id.as_str()), nodes, warnings)?;
        child_ids.push(child_id);
    }
    nodes[node_index].children = child_ids;

    Ok(id)
}

fn required_string(
    node: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<String, NormalizationError> {
    node.get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            NormalizationError::InvalidPayloadField(format!("node.{field} must be a string"))
        })
}

fn map_node_kind(
    node_type: Option<&str>,
    node_id: &str,
    warnings: &mut Vec<NormalizationWarning>,
) -> NodeKind {
    match node_type.unwrap_or("UNKNOWN") {
        "FRAME" => NodeKind::Frame,
        "GROUP" => NodeKind::Group,
        "COMPONENT" => NodeKind::Component,
        "INSTANCE" => NodeKind::Instance,
        "COMPONENT_SET" => NodeKind::ComponentSet,
        "TEXT" => NodeKind::Text,
        "RECTANGLE" => NodeKind::Rectangle,
        "ELLIPSE" => NodeKind::Ellipse,
        "VECTOR" => NodeKind::Vector,
        other => {
            warnings.push(NormalizationWarning {
                code: "UNSUPPORTED_NODE_TYPE".to_string(),
                message: format!("unsupported node type `{other}` normalized as `unknown`"),
                node_id: Some(node_id.to_string()),
            });
            NodeKind::Unknown
        }
    }
}

fn parse_bounds(bounds_value: Option<&Value>) -> Result<Bounds, NormalizationError> {
    let Some(bounds) = bounds_value else {
        return Ok(Bounds {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        });
    };

    let object = bounds.as_object().ok_or_else(|| {
        NormalizationError::InvalidPayloadField(
            "node.absoluteBoundingBox must be a JSON object".to_string(),
        )
    })?;

    Ok(Bounds {
        x: required_f32(object, "x", "node.absoluteBoundingBox.x")?,
        y: required_f32(object, "y", "node.absoluteBoundingBox.y")?,
        w: required_f32(object, "width", "node.absoluteBoundingBox.width")?,
        h: required_f32(object, "height", "node.absoluteBoundingBox.height")?,
    })
}

fn required_f32(
    object: &serde_json::Map<String, Value>,
    field: &'static str,
    description: &'static str,
) -> Result<f32, NormalizationError> {
    object
        .get(field)
        .and_then(Value::as_f64)
        .map(|number| number as f32)
        .ok_or_else(|| {
            NormalizationError::InvalidPayloadField(format!("{description} must be a number"))
        })
}

fn parse_children(children_value: Option<&Value>) -> Result<Vec<&Value>, NormalizationError> {
    let Some(value) = children_value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .map(|children| children.iter().collect::<Vec<_>>())
        .ok_or_else(|| {
            NormalizationError::InvalidPayloadField("node.children must be an array".to_string())
        })
}

fn append_unsupported_field_warnings(
    node: &serde_json::Map<String, Value>,
    node_id: &str,
    warnings: &mut Vec<NormalizationWarning>,
) {
    const SUPPORTED_FIELDS: [&str; 6] = [
        "id",
        "name",
        "type",
        "visible",
        "absoluteBoundingBox",
        "children",
    ];

    let mut unsupported_fields = node
        .keys()
        .filter(|field| !SUPPORTED_FIELDS.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unsupported_fields.sort();

    for field in unsupported_fields {
        warnings.push(NormalizationWarning {
            code: "UNSUPPORTED_NODE_FIELD".to_string(),
            message: format!("unsupported field `{field}` ignored during normalization"),
            node_id: Some(node_id.to_string()),
        });
    }
}

fn default_style() -> NodeStyle {
    NodeStyle {
        opacity: 1.0,
        corner_radius: None,
        fills: Vec::new(),
        strokes: Vec::new(),
    }
}

fn default_component() -> ComponentMetadata {
    ComponentMetadata {
        component_id: None,
        component_set_id: None,
        instance_of: None,
        variant_properties: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedDocument {
    pub schema_version: String,
    pub source: NormalizedSource,
    pub nodes: Vec<NormalizedNode>,
}

impl Default for NormalizedDocument {
    fn default() -> Self {
        Self {
            schema_version: NORMALIZED_SCHEMA_VERSION.to_string(),
            source: NormalizedSource::default(),
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedSource {
    pub file_key: String,
    pub root_node_id: String,
    pub figma_api_version: String,
}

impl Default for NormalizedSource {
    fn default() -> Self {
        Self {
            file_key: String::new(),
            root_node_id: String::new(),
            figma_api_version: FIGMA_API_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: NodeKind,
    pub visible: bool,
    pub bounds: Bounds,
    pub layout: Option<LayoutMetadata>,
    pub constraints: Option<LayoutConstraints>,
    pub style: NodeStyle,
    pub component: ComponentMetadata,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Frame,
    Group,
    Component,
    Instance,
    ComponentSet,
    Text,
    Rectangle,
    Ellipse,
    Vector,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutMetadata {
    pub mode: LayoutMode,
    pub primary_align: Align,
    pub cross_align: Align,
    pub item_spacing: f32,
    pub padding: Padding,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutMode {
    None,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutConstraints {
    pub horizontal: ConstraintMode,
    pub vertical: ConstraintMode,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintMode {
    Min,
    Max,
    Stretch,
    Center,
    Scale,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodeStyle {
    pub opacity: f32,
    pub corner_radius: Option<f32>,
    pub fills: Vec<Paint>,
    pub strokes: Vec<Stroke>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Paint {
    pub kind: PaintKind,
    pub color: Option<Color>,
    pub image_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaintKind {
    Solid,
    Image,
    Gradient,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stroke {
    pub width: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentMetadata {
    pub component_id: Option<String>,
    pub component_set_id: Option<String>,
    pub instance_of: Option<String>,
    pub variant_properties: Vec<VariantProperty>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VariantProperty {
    pub name: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn normalized_document_round_trip() {
        let doc = sample_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let back: NormalizedDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, back);
    }

    #[test]
    fn children_order_is_stable() {
        let doc = sample_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let back: NormalizedDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.nodes[0].children,
            vec!["2:1".to_string(), "3:1".to_string()]
        );
    }

    #[test]
    fn node_collection_order_is_stable() {
        let doc = sample_document();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let back: NormalizedDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.nodes
                .iter()
                .map(|node| node.id.clone())
                .collect::<Vec<_>>(),
            vec!["1:1".to_string(), "2:1".to_string(), "3:1".to_string()]
        );
    }

    #[test]
    fn root_contract_fields_are_explicit() {
        let doc = sample_document();
        let json = serde_json::to_value(&doc).unwrap();

        let object = json
            .as_object()
            .expect("normalized document should serialize as an object");
        assert_eq!(
            object.get("schema_version"),
            Some(&Value::String(NORMALIZED_SCHEMA_VERSION.to_string()))
        );
        assert!(object.contains_key("source"));
        assert!(object.contains_key("nodes"));

        let source = object
            .get("source")
            .and_then(Value::as_object)
            .expect("source should serialize as an object");
        assert_eq!(
            source.get("file_key"),
            Some(&Value::String("abc123".to_string()))
        );
        assert_eq!(
            source.get("root_node_id"),
            Some(&Value::String("1:1".to_string()))
        );
        assert_eq!(
            source.get("figma_api_version"),
            Some(&Value::String(FIGMA_API_VERSION.to_string()))
        );
    }

    #[test]
    fn defaults_include_explicit_versions() {
        let doc = NormalizedDocument::default();
        assert_eq!(doc.schema_version, NORMALIZED_SCHEMA_VERSION);
        assert_eq!(doc.source.figma_api_version, FIGMA_API_VERSION);
    }

    #[test]
    fn normalize_snapshot_maps_minimal_document_tree() {
        let request = figma_client::FetchNodesRequest::new("abc123".to_string(), "1:1".to_string())
            .expect("request should be valid");
        let snapshot = figma_client::fetch_snapshot_from_fixture(
            &request,
            r#"{
                "document": {
                    "id": "1:1",
                    "name": "Root",
                    "type": "FRAME",
                    "visible": true,
                    "absoluteBoundingBox": { "x": 0.0, "y": 0.0, "width": 390.0, "height": 844.0 },
                    "children": [
                        {
                            "id": "2:1",
                            "name": "Title",
                            "type": "TEXT",
                            "visible": true,
                            "absoluteBoundingBox": { "x": 20.0, "y": 24.0, "width": 140.0, "height": 40.0 },
                            "children": []
                        }
                    ]
                }
            }"#,
        )
        .expect("fixture should parse");

        let output = super::normalize_snapshot(&snapshot).expect("snapshot should normalize");
        assert!(output.warnings.is_empty());
        assert_eq!(output.document.source.file_key, "abc123");
        assert_eq!(output.document.source.root_node_id, "1:1");
        assert_eq!(output.document.nodes.len(), 2);
        assert_eq!(
            output
                .document
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["1:1", "2:1"]
        );
        assert_eq!(output.document.nodes[0].children, vec!["2:1".to_string()]);
        assert_eq!(output.document.nodes[1].children, Vec::<String>::new());
    }

    #[test]
    fn normalize_snapshot_emits_warning_for_unsupported_fields() {
        let request = figma_client::FetchNodesRequest::new("abc123".to_string(), "1:1".to_string())
            .expect("request should be valid");
        let snapshot = figma_client::fetch_snapshot_from_fixture(
            &request,
            r#"{
                "document": {
                    "id": "1:1",
                    "name": "Root",
                    "type": "FRAME",
                    "visible": true,
                    "blendMode": "MULTIPLY",
                    "absoluteBoundingBox": { "x": 0.0, "y": 0.0, "width": 390.0, "height": 844.0 },
                    "children": []
                }
            }"#,
        )
        .expect("fixture should parse");

        let output = super::normalize_snapshot(&snapshot).expect("snapshot should normalize");
        assert_eq!(output.warnings.len(), 1);
        assert_eq!(output.warnings[0].code, "UNSUPPORTED_NODE_FIELD");
        assert_eq!(output.warnings[0].node_id.as_deref(), Some("1:1"));
        assert!(output.warnings[0].message.contains("blendMode"));
    }

    #[test]
    fn normalize_snapshot_rejects_missing_document_payload() {
        let request = figma_client::FetchNodesRequest::new("abc123".to_string(), "1:1".to_string())
            .expect("request should be valid");
        let snapshot = figma_client::fetch_snapshot_from_fixture(&request, r#"{"components":{}}"#)
            .expect("fixture should parse");

        let err = super::normalize_snapshot(&snapshot).expect_err("missing document should fail");
        assert!(
            err.to_string()
                .contains("missing required payload field: document")
        );
    }

    fn sample_document() -> NormalizedDocument {
        NormalizedDocument {
            schema_version: NORMALIZED_SCHEMA_VERSION.to_string(),
            source: NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: FIGMA_API_VERSION.to_string(),
            },
            nodes: vec![
                NormalizedNode {
                    id: "1:1".to_string(),
                    parent_id: None,
                    name: "Root".to_string(),
                    kind: NodeKind::Frame,
                    visible: true,
                    bounds: Bounds {
                        x: 0.0,
                        y: 0.0,
                        w: 390.0,
                        h: 844.0,
                    },
                    layout: Some(LayoutMetadata {
                        mode: LayoutMode::Vertical,
                        primary_align: Align::Start,
                        cross_align: Align::Stretch,
                        item_spacing: 16.0,
                        padding: Padding {
                            top: 24.0,
                            right: 20.0,
                            bottom: 24.0,
                            left: 20.0,
                        },
                    }),
                    constraints: Some(LayoutConstraints {
                        horizontal: ConstraintMode::Stretch,
                        vertical: ConstraintMode::Min,
                    }),
                    style: NodeStyle {
                        opacity: 1.0,
                        corner_radius: Some(12.0),
                        fills: vec![Paint {
                            kind: PaintKind::Solid,
                            color: Some(Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            image_ref: None,
                        }],
                        strokes: vec![Stroke {
                            width: 1.0,
                            color: Color {
                                r: 0.9,
                                g: 0.9,
                                b: 0.9,
                                a: 1.0,
                            },
                        }],
                    },
                    component: ComponentMetadata {
                        component_id: None,
                        component_set_id: None,
                        instance_of: None,
                        variant_properties: vec![VariantProperty {
                            name: "state".to_string(),
                            value: "default".to_string(),
                        }],
                    },
                    children: vec!["2:1".to_string(), "3:1".to_string()],
                },
                NormalizedNode {
                    id: "2:1".to_string(),
                    parent_id: Some("1:1".to_string()),
                    name: "Title".to_string(),
                    kind: NodeKind::Text,
                    visible: true,
                    bounds: Bounds {
                        x: 20.0,
                        y: 24.0,
                        w: 160.0,
                        h: 38.0,
                    },
                    layout: None,
                    constraints: Some(LayoutConstraints {
                        horizontal: ConstraintMode::Stretch,
                        vertical: ConstraintMode::Min,
                    }),
                    style: NodeStyle {
                        opacity: 1.0,
                        corner_radius: None,
                        fills: vec![Paint {
                            kind: PaintKind::Solid,
                            color: Some(Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            image_ref: None,
                        }],
                        strokes: Vec::new(),
                    },
                    component: ComponentMetadata {
                        component_id: None,
                        component_set_id: None,
                        instance_of: None,
                        variant_properties: Vec::new(),
                    },
                    children: Vec::new(),
                },
                NormalizedNode {
                    id: "3:1".to_string(),
                    parent_id: Some("1:1".to_string()),
                    name: "PrimaryButton".to_string(),
                    kind: NodeKind::Instance,
                    visible: true,
                    bounds: Bounds {
                        x: 20.0,
                        y: 78.0,
                        w: 350.0,
                        h: 48.0,
                    },
                    layout: None,
                    constraints: Some(LayoutConstraints {
                        horizontal: ConstraintMode::Stretch,
                        vertical: ConstraintMode::Min,
                    }),
                    style: NodeStyle {
                        opacity: 1.0,
                        corner_radius: Some(8.0),
                        fills: vec![Paint {
                            kind: PaintKind::Solid,
                            color: Some(Color {
                                r: 0.14,
                                g: 0.45,
                                b: 0.95,
                                a: 1.0,
                            }),
                            image_ref: None,
                        }],
                        strokes: Vec::new(),
                    },
                    component: ComponentMetadata {
                        component_id: Some("42:7".to_string()),
                        component_set_id: Some("42:0".to_string()),
                        instance_of: Some("42:7".to_string()),
                        variant_properties: vec![VariantProperty {
                            name: "state".to_string(),
                            value: "enabled".to_string(),
                        }],
                    },
                    children: Vec::new(),
                },
            ],
        }
    }
}

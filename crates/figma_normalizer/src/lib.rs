#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NormalizedDocument {
    pub schema_version: String,
    pub source: NormalizedSource,
    pub nodes: Vec<NormalizedNode>,
}

impl Default for NormalizedDocument {
    fn default() -> Self {
        Self {
            schema_version: "1.0".to_string(),
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
            figma_api_version: "v1".to_string(),
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
        assert_eq!(back.nodes[0].children, vec!["2:1".to_string(), "3:1".to_string()]);
    }

    fn sample_document() -> NormalizedDocument {
        NormalizedDocument {
            schema_version: "1.0".to_string(),
            source: NormalizedSource {
                file_key: "abc123".to_string(),
                root_node_id: "1:1".to_string(),
                figma_api_version: "v1".to_string(),
            },
            nodes: vec![NormalizedNode {
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
            }],
        }
    }
}

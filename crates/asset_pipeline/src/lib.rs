#![forbid(unsafe_code)]

pub const ASSET_MANIFEST_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetManifest {
    pub manifest_version: String,
    pub generation: GenerationMetadata,
    pub assets: Vec<AssetEntry>,
    pub warnings: Vec<AssetExportWarning>,
}

impl Default for AssetManifest {
    fn default() -> Self {
        Self {
            manifest_version: ASSET_MANIFEST_VERSION.to_string(),
            generation: GenerationMetadata::default(),
            assets: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn build_asset_manifest(normalized: &figma_normalizer::NormalizationOutput) -> AssetManifest {
    let mut assets = Vec::new();
    let mut warnings = Vec::new();

    for node in &normalized.document.nodes {
        let image_fills = node
            .style
            .fills
            .iter()
            .filter(|fill| fill.kind == figma_normalizer::PaintKind::Image)
            .collect::<Vec<_>>();

        for fill in image_fills {
            match fill.image_ref.as_ref() {
                Some(image_ref) => assets.push(AssetEntry {
                    node_id: node.id.clone(),
                    image_ref: Some(image_ref.clone()),
                    hashed_output_filename: format!(
                        "img_{}.png",
                        sanitize_identifier(node.id.as_str())
                    ),
                    format: AssetFormat::Png,
                    width_px: node.bounds.w.max(0.0).round() as u32,
                    height_px: node.bounds.h.max(0.0).round() as u32,
                    dedupe_key: format!("node-{}", sanitize_identifier(node.id.as_str())),
                }),
                None => warnings.push(AssetExportWarning {
                    code: "MISSING_IMAGE_REF".to_string(),
                    message: "Image fill had no image_ref and was skipped.".to_string(),
                    node_id: Some(node.id.clone()),
                    fallback_applied: false,
                }),
            }
        }
    }

    assets.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.image_ref.cmp(&right.image_ref))
    });

    AssetManifest {
        manifest_version: ASSET_MANIFEST_VERSION.to_string(),
        generation: GenerationMetadata {
            source_file_key: normalized.document.source.file_key.clone(),
            generator_version: "0.1.0".to_string(),
        },
        assets,
        warnings,
    }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GenerationMetadata {
    pub source_file_key: String,
    pub generator_version: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetEntry {
    pub node_id: String,
    pub image_ref: Option<String>,
    #[serde(alias = "output_filename")]
    pub hashed_output_filename: String,
    pub format: AssetFormat,
    pub width_px: u32,
    pub height_px: u32,
    pub dedupe_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetFormat {
    Png,
    Jpeg,
    Pdf,
    Svg,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetExportWarning {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
    pub fallback_applied: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn asset_manifest_round_trip() {
        let manifest = sample_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let back: AssetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, back);
    }

    #[test]
    fn asset_entry_order_is_stable() {
        let manifest = sample_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let back: AssetManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(
            back.assets
                .iter()
                .map(|asset| asset.hashed_output_filename.clone())
                .collect::<Vec<_>>(),
            vec![
                "img_primary_button.png".to_string(),
                "img_logo_mark.pdf".to_string()
            ]
        );
    }

    #[test]
    fn default_manifest_uses_expected_contract_version() {
        let manifest = AssetManifest::default();
        assert_eq!(manifest.manifest_version, ASSET_MANIFEST_VERSION);
        assert!(manifest.generation.source_file_key.is_empty());
        assert!(manifest.generation.generator_version.is_empty());
        assert!(manifest.assets.is_empty());
    }

    #[test]
    fn asset_entry_field_order_is_deterministic() {
        let entry = sample_manifest().assets[0].clone();
        let json = serde_json::to_string(&entry).unwrap();

        assert_eq!(
            json,
            "{\"node_id\":\"10:1\",\"image_ref\":\"figma-image-ref-1\",\"hashed_output_filename\":\"img_primary_button.png\",\"format\":\"png\",\"width_px\":240,\"height_px\":64,\"dedupe_key\":\"hash-aaa111\"}"
        );
    }

    #[test]
    fn manifest_contract_matches_next_stage_map() {
        let manifest = sample_manifest();
        let json = serde_json::to_value(&manifest).unwrap();

        assert_eq!(
            json,
            json!({
                "manifest_version": "1.0",
                "generation": {
                    "source_file_key": "abc123",
                    "generator_version": "0.1.0",
                },
                "assets": [
                    {
                        "node_id": "10:1",
                        "image_ref": "figma-image-ref-1",
                        "hashed_output_filename": "img_primary_button.png",
                        "format": "png",
                        "width_px": 240,
                        "height_px": 64,
                        "dedupe_key": "hash-aaa111",
                    },
                    {
                        "node_id": "12:3",
                        "image_ref": "figma-image-ref-2",
                        "hashed_output_filename": "img_logo_mark.pdf",
                        "format": "pdf",
                        "width_px": 128,
                        "height_px": 128,
                        "dedupe_key": "hash-bbb222",
                    }
                ],
                "warnings": [
                    {
                        "code": "FORMAT_FALLBACK",
                        "message": "SVG export unavailable; fell back to PDF.",
                        "node_id": "12:3",
                        "fallback_applied": true
                    }
                ]
            })
        );
    }

    #[test]
    fn build_asset_manifest_extracts_image_fill_assets_deterministically() {
        let normalized = figma_normalizer::NormalizationOutput {
            document: figma_normalizer::NormalizedDocument {
                schema_version: figma_normalizer::NORMALIZED_SCHEMA_VERSION.to_string(),
                source: figma_normalizer::NormalizedSource {
                    file_key: "abc123".to_string(),
                    root_node_id: "1:1".to_string(),
                    figma_api_version: figma_normalizer::FIGMA_API_VERSION.to_string(),
                },
                nodes: vec![
                    image_node("10:1", "figma-image-ref-1", 240.0, 64.0),
                    image_node("12:3", "figma-image-ref-2", 128.0, 128.0),
                ],
            },
            warnings: Vec::new(),
        };
        let manifest = super::build_asset_manifest(&normalized);
        assert_eq!(manifest.manifest_version, super::ASSET_MANIFEST_VERSION);
        assert_eq!(manifest.generation.source_file_key, "abc123");
        assert_eq!(manifest.generation.generator_version, "0.1.0");
        assert_eq!(manifest.assets.len(), 2);
        assert_eq!(manifest.assets[0].node_id, "10:1");
        assert_eq!(manifest.assets[1].node_id, "12:3");
        assert_eq!(manifest.assets[0].format, super::AssetFormat::Png);
        assert_eq!(manifest.assets[0].width_px, 240);
        assert_eq!(manifest.assets[0].height_px, 64);
    }

    fn image_node(
        id: &str,
        image_ref: &str,
        width: f32,
        height: f32,
    ) -> figma_normalizer::NormalizedNode {
        figma_normalizer::NormalizedNode {
            id: id.to_string(),
            parent_id: Some("1:1".to_string()),
            name: "Image".to_string(),
            kind: figma_normalizer::NodeKind::Rectangle,
            visible: true,
            bounds: figma_normalizer::Bounds {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            },
            layout: None,
            constraints: None,
            style: figma_normalizer::NodeStyle {
                opacity: 1.0,
                corner_radius: None,
                fills: vec![figma_normalizer::Paint {
                    kind: figma_normalizer::PaintKind::Image,
                    color: None,
                    image_ref: Some(image_ref.to_string()),
                }],
                strokes: Vec::new(),
            },
            component: figma_normalizer::ComponentMetadata {
                component_id: None,
                component_set_id: None,
                instance_of: None,
                variant_properties: Vec::new(),
            },
            passthrough_fields: std::collections::BTreeMap::new(),
            children: Vec::new(),
        }
    }

    fn sample_manifest() -> AssetManifest {
        AssetManifest {
            manifest_version: ASSET_MANIFEST_VERSION.to_string(),
            generation: GenerationMetadata {
                source_file_key: "abc123".to_string(),
                generator_version: "0.1.0".to_string(),
            },
            assets: vec![
                AssetEntry {
                    node_id: "10:1".to_string(),
                    image_ref: Some("figma-image-ref-1".to_string()),
                    hashed_output_filename: "img_primary_button.png".to_string(),
                    format: AssetFormat::Png,
                    width_px: 240,
                    height_px: 64,
                    dedupe_key: "hash-aaa111".to_string(),
                },
                AssetEntry {
                    node_id: "12:3".to_string(),
                    image_ref: Some("figma-image-ref-2".to_string()),
                    hashed_output_filename: "img_logo_mark.pdf".to_string(),
                    format: AssetFormat::Pdf,
                    width_px: 128,
                    height_px: 128,
                    dedupe_key: "hash-bbb222".to_string(),
                },
            ],
            warnings: vec![AssetExportWarning {
                code: "FORMAT_FALLBACK".to_string(),
                message: "SVG export unavailable; fell back to PDF.".to_string(),
                node_id: Some("12:3".to_string()),
                fallback_applied: true,
            }],
        }
    }
}

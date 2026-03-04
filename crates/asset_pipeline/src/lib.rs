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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GenerationMetadata {
    pub source_file_key: String,
    pub generator_version: String,
}

impl Default for GenerationMetadata {
    fn default() -> Self {
        Self {
            source_file_key: String::new(),
            generator_version: String::new(),
        }
    }
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

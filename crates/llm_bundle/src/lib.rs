#![forbid(unsafe_code)]

use std::path::Path;

pub const LLM_BUNDLE_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmBundle {
    pub bundle_version: String,
    pub target: String,
    pub prompt_template_version: String,
    pub ui_spec: BundleInputRef,
    pub asset_manifest: BundleInputRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings_summary: Vec<BundleWarningSummary>,
}

impl LlmBundle {
    pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleInputRef {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleWarningSummary {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmBundleError {
    #[error("target is required")]
    MissingTarget,
    #[error("prompt_template_version is required")]
    MissingPromptTemplateVersion,
    #[error("failed to read input `{path}`: {source}")]
    ReadInput {
        path: String,
        source: std::io::Error,
    },
}

pub fn build_bundle(
    target: &str,
    ui_spec_path: &Path,
    asset_manifest_path: &Path,
    warnings_summary: Vec<BundleWarningSummary>,
    prompt_template_version: &str,
) -> Result<LlmBundle, LlmBundleError> {
    let target = target.trim();
    if target.is_empty() {
        return Err(LlmBundleError::MissingTarget);
    }

    let prompt_template_version = prompt_template_version.trim();
    if prompt_template_version.is_empty() {
        return Err(LlmBundleError::MissingPromptTemplateVersion);
    }

    let ui_spec = bundle_input_ref(ui_spec_path)?;
    let asset_manifest = bundle_input_ref(asset_manifest_path)?;

    Ok(LlmBundle {
        bundle_version: LLM_BUNDLE_VERSION.to_string(),
        target: target.to_string(),
        prompt_template_version: prompt_template_version.to_string(),
        ui_spec,
        asset_manifest,
        warnings_summary,
    })
}

fn bundle_input_ref(path: &Path) -> Result<BundleInputRef, LlmBundleError> {
    let bytes = std::fs::read(path).map_err(|source| LlmBundleError::ReadInput {
        path: path.display().to_string(),
        source,
    })?;

    Ok(BundleInputRef {
        path: path.display().to_string(),
        sha256: sha256_hex(bytes.as_slice()),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_produce_same_hashes() {
        let workspace_root = unique_test_workspace_root("same_inputs_produce_same_hashes");
        let ui_spec_path = workspace_root.join("ui_spec.json");
        let assets_path = workspace_root.join("asset_manifest.json");
        std::fs::write(
            &ui_spec_path,
            "{\n  \"spec_version\": \"1.0\",\n  \"source\": {\"file_key\": \"abc123\", \"root_node_id\": \"0:1\", \"generator_version\": \"0.1.0\"},\n  \"root\": {\"id\": \"0:1\", \"name\": \"Root\", \"kind\": \"container\", \"layout\": {\"strategy\": \"absolute\", \"item_spacing\": 0.0}, \"style\": {\"opacity\": 1.0, \"corner_radius\": null}, \"children\": []},\n  \"warnings\": []\n}\n",
        )
        .unwrap();
        std::fs::write(
            &assets_path,
            "{\n  \"manifest_version\": \"1.0\",\n  \"assets\": []\n}\n",
        )
        .unwrap();

        let first = build_bundle(
            "swiftui",
            ui_spec_path.as_path(),
            assets_path.as_path(),
            Vec::new(),
            "v1",
        )
        .unwrap();
        let second = build_bundle(
            "swiftui",
            ui_spec_path.as_path(),
            assets_path.as_path(),
            Vec::new(),
            "v1",
        )
        .unwrap();

        assert_eq!(first.ui_spec.sha256, second.ui_spec.sha256);
        assert_eq!(first.asset_manifest.sha256, second.asset_manifest.sha256);

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn warning_summary_is_preserved() {
        let workspace_root = unique_test_workspace_root("warning_summary_is_preserved");
        let ui_spec_path = workspace_root.join("ui_spec.json");
        let assets_path = workspace_root.join("asset_manifest.json");
        std::fs::write(
            &ui_spec_path,
            "{\n  \"spec_version\": \"1.0\",\n  \"source\": {\"file_key\": \"abc123\", \"root_node_id\": \"0:1\", \"generator_version\": \"0.1.0\"},\n  \"root\": {\"id\": \"0:1\", \"name\": \"Root\", \"kind\": \"container\", \"layout\": {\"strategy\": \"absolute\", \"item_spacing\": 0.0}, \"style\": {\"opacity\": 1.0, \"corner_radius\": null}, \"children\": []},\n  \"warnings\": []\n}\n",
        )
        .unwrap();
        std::fs::write(
            &assets_path,
            "{\n  \"manifest_version\": \"1.0\",\n  \"assets\": []\n}\n",
        )
        .unwrap();

        let warnings = vec![
            BundleWarningSummary {
                code: "LOW_CONFIDENCE_LAYOUT".to_string(),
                node_id: Some("123:456".to_string()),
            },
            BundleWarningSummary {
                code: "UNSUPPORTED_BLEND_MODE".to_string(),
                node_id: None,
            },
        ];
        let bundle = build_bundle(
            "swiftui",
            ui_spec_path.as_path(),
            assets_path.as_path(),
            warnings.clone(),
            "v1",
        )
        .unwrap();

        assert_eq!(bundle.warnings_summary, warnings);

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    fn unique_test_workspace_root(test_name: &str) -> std::path::PathBuf {
        let timestamp_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "forge-llm-bundle-{test_name}-{}-{timestamp_nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
        path
    }
}

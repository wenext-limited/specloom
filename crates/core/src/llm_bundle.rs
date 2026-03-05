pub const LLM_BUNDLE_VERSION: &str = "llm_bundle/1.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmBundle {
    pub version: String,
    pub request: BundleRequest,
    pub figma: BundleFigmaContext,
    pub artifacts: BundleArtifacts,
    pub instructions: BundleInstructions,
    pub tool_contract: BundleToolContract,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleRequest {
    pub target: String,
    pub intent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleFigmaContext {
    pub source_url: String,
    pub file_key: String,
    pub root_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleArtifacts {
    pub ui_spec: BundleArtifactRef,
    pub agent_context: BundleArtifactRef,
    pub search_index: BundleArtifactRef,
    pub asset_manifest: BundleArtifactRef,
    pub root_screenshot: Option<BundleArtifactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleArtifactRef {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleInstructions {
    pub skills_guide_markdown: String,
    pub agent_playbook_markdown: String,
    pub figma_ui_coder_markdown: String,
    pub skill_docs: Vec<BundleSkillDoc>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleSkillDoc {
    pub name: String,
    pub path: String,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleToolContract {
    pub tools: Vec<BundleToolDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleToolDefinition {
    pub name: String,
    pub usage: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_bundle_round_trip_json() {
        let bundle = sample_bundle();
        let bytes = serde_json::to_vec_pretty(&bundle).expect("bundle should encode");
        let decoded: LlmBundle = serde_json::from_slice(bytes.as_slice()).expect("bundle should decode");
        assert_eq!(decoded, bundle);
    }

    #[test]
    fn llm_bundle_contract_version_is_explicit() {
        assert_eq!(LLM_BUNDLE_VERSION, "llm_bundle/1.0");
        assert_eq!(sample_bundle().version, "llm_bundle/1.0");
    }

    fn sample_bundle() -> LlmBundle {
        LlmBundle {
            version: LLM_BUNDLE_VERSION.to_string(),
            request: BundleRequest {
                target: "react-tailwind".to_string(),
                intent: "Generate login page".to_string(),
            },
            figma: BundleFigmaContext {
                source_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
                file_key: "abc".to_string(),
                root_node_id: "1:2".to_string(),
            },
            artifacts: BundleArtifacts {
                ui_spec: sample_artifact_ref("output/specs/ui_spec.ron"),
                agent_context: sample_artifact_ref("output/agent/agent_context.json"),
                search_index: sample_artifact_ref("output/agent/search_index.json"),
                asset_manifest: sample_artifact_ref("output/assets/asset_manifest.json"),
                root_screenshot: Some(sample_artifact_ref("output/images/root_1_2.png")),
            },
            instructions: BundleInstructions {
                skills_guide_markdown: "# skills".to_string(),
                agent_playbook_markdown: "# playbook".to_string(),
                figma_ui_coder_markdown: "# figma-ui-coder".to_string(),
                skill_docs: vec![
                    BundleSkillDoc {
                        name: "authoring-transform-plan".to_string(),
                        path: ".codex/skills/authoring-transform-plan/SKILL.md".to_string(),
                        markdown: "# authoring transform plan".to_string(),
                    },
                    BundleSkillDoc {
                        name: "generating-ui-from-ui-spec-ron".to_string(),
                        path: ".codex/skills/generating-ui-from-ui-spec-ron/SKILL.md".to_string(),
                        markdown: "# generating ui".to_string(),
                    },
                ],
            },
            tool_contract: BundleToolContract {
                tools: vec![
                    BundleToolDefinition {
                        name: "find_nodes".to_string(),
                        usage: "specloom agent-tool find-nodes --query \"...\" --output json"
                            .to_string(),
                    },
                    BundleToolDefinition {
                        name: "get_node_info".to_string(),
                        usage: "specloom agent-tool get-node-info --node-id <NODE_ID> --output json"
                            .to_string(),
                    },
                ],
            },
        }
    }

    fn sample_artifact_ref(path: &str) -> BundleArtifactRef {
        BundleArtifactRef {
            path: path.to_string(),
            sha256: "6f7f0f3f7f5872a76fdd8f36b7434eb95ebf7797b690ce5f5a3e87f1465d913d"
                .to_string(),
        }
    }
}

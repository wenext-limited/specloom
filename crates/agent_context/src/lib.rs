#![forbid(unsafe_code)]

mod search;

pub use search::{
    SearchMatch, SearchResult, SearchStatus, classify_status, normalize_tokens, rank_candidates,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentContext {
    pub version: String,
    pub screen: ScreenRef,
    pub rules: GenerationRules,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skeleton: Vec<SkeletonNode>,
}

impl AgentContext {
    pub fn to_pretty_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    pub fn sample() -> Self {
        Self {
            version: "agent_context/1.0".to_string(),
            screen: ScreenRef {
                root_node_id: "1:2".to_string(),
                root_screenshot_ref: "output/images/root_1_2.png".to_string(),
            },
            rules: GenerationRules {
                on_node_mismatch: "warn_and_continue".to_string(),
            },
            tools: vec![
                "find_nodes".to_string(),
                "get_node_info".to_string(),
                "get_node_screenshot".to_string(),
                "get_asset".to_string(),
            ],
            skeleton: vec![SkeletonNode {
                node_id: "1:10".to_string(),
                node_type: "FRAME".to_string(),
                name: "Header".to_string(),
                path: "Main/Header".to_string(),
                children: vec![],
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenRef {
    pub root_node_id: String,
    pub root_screenshot_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationRules {
    pub on_node_mismatch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonNode {
    pub node_id: String,
    pub node_type: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchIndex {
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<SearchIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchIndexEntry {
    pub node_id: String,
    pub name: String,
    pub node_type: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_tokens: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub normalized_tokens: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geometry_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationWarnings {
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<GenerationWarning>,
}

impl GenerationWarnings {
    pub fn sample() -> Self {
        Self {
            version: "generation_warnings/1.0".to_string(),
            warnings: vec![GenerationWarning {
                warning_id: "warn-1".to_string(),
                warning_type: "NODE_NOT_FOUND".to_string(),
                severity: "warning".to_string(),
                node_query: "welcome back".to_string(),
                candidate_node_ids: vec![],
                agent_action: "continue_with_best_effort".to_string(),
                message: "No node candidate found for query".to_string(),
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationWarning {
    pub warning_id: String,
    pub warning_type: String,
    pub severity: String,
    pub node_query: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_node_ids: Vec<String>,
    pub agent_action: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationTrace {
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceEvent {
    pub event_id: String,
    pub tool_name: String,
    pub status: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_node_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_context_round_trip_json() {
        let context = AgentContext::sample();
        let encoded = context.to_pretty_json().expect("context should serialize");
        let decoded: AgentContext =
            serde_json::from_slice(encoded.as_slice()).expect("context should deserialize");
        assert_eq!(decoded, context);
    }

    #[test]
    fn warning_file_round_trip_json() {
        let report = GenerationWarnings::sample();
        let encoded = serde_json::to_vec_pretty(&report).expect("report should serialize");
        let decoded: GenerationWarnings =
            serde_json::from_slice(encoded.as_slice()).expect("report should deserialize");
        assert_eq!(decoded, report);
    }
}

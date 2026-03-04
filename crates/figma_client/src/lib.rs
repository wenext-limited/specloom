#![forbid(unsafe_code)]

use serde_json::Value;

pub const RAW_SNAPSHOT_SCHEMA_VERSION: &str = "1.0";
pub const FIGMA_API_VERSION: &str = "v1";

#[derive(Debug, thiserror::Error)]
pub enum FetchClientError {
    #[error("invalid fetch request: {0}")]
    InvalidRequest(String),
    #[error("invalid fixture json: {0}")]
    InvalidFixtureJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchNodesRequest {
    pub file_key: String,
    pub node_id: String,
}

impl FetchNodesRequest {
    pub fn new(file_key: String, node_id: String) -> Result<Self, FetchClientError> {
        let file_key = file_key.trim().to_string();
        if file_key.is_empty() {
            return Err(FetchClientError::InvalidRequest(
                "file_key is required".to_string(),
            ));
        }

        let node_id = node_id.trim().to_string();
        if node_id.is_empty() {
            return Err(FetchClientError::InvalidRequest(
                "node_id is required".to_string(),
            ));
        }

        Ok(Self { file_key, node_id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSnapshotSource {
    pub file_key: String,
    pub node_id: String,
    pub figma_api_version: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFigmaSnapshot {
    pub snapshot_version: String,
    pub source: RawSnapshotSource,
    pub payload: Value,
}

pub fn fetch_snapshot_from_fixture(
    request: &FetchNodesRequest,
    fixture_json: &str,
) -> Result<RawFigmaSnapshot, FetchClientError> {
    let payload: Value = serde_json::from_str(fixture_json)?;
    Ok(RawFigmaSnapshot {
        snapshot_version: RAW_SNAPSHOT_SCHEMA_VERSION.to_string(),
        source: RawSnapshotSource {
            file_key: request.file_key.clone(),
            node_id: request.node_id.clone(),
            figma_api_version: FIGMA_API_VERSION.to_string(),
        },
        payload,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn fetch_nodes_request_rejects_missing_file_key() {
        let err = super::FetchNodesRequest::new("".to_string(), "123:456".to_string())
            .expect_err("empty file key should be rejected");
        assert_eq!(
            err.to_string(),
            "invalid fetch request: file_key is required"
        );
    }

    #[test]
    fn fetch_snapshot_from_fixture_preserves_source_and_payload() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let fixture = r#"{
            "document": {
                "id": "123:456",
                "name": "Root Frame"
            }
        }"#;

        let snapshot = super::fetch_snapshot_from_fixture(&request, fixture)
            .expect("fixture payload should parse");

        assert_eq!(
            snapshot.snapshot_version,
            super::RAW_SNAPSHOT_SCHEMA_VERSION
        );
        assert_eq!(snapshot.source.file_key, "abc123");
        assert_eq!(snapshot.source.node_id, "123:456");
        assert_eq!(snapshot.source.figma_api_version, super::FIGMA_API_VERSION);
        assert_eq!(
            snapshot.payload,
            json!({
                "document": {
                    "id": "123:456",
                    "name": "Root Frame"
                }
            })
        );
    }

    #[test]
    fn fetch_snapshot_from_fixture_reports_invalid_json() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let err = super::fetch_snapshot_from_fixture(&request, "{")
            .expect_err("malformed fixture should fail");
        assert!(err.to_string().starts_with("invalid fixture json:"));
    }

    #[test]
    fn raw_snapshot_contract_round_trip() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");
        let snapshot = super::fetch_snapshot_from_fixture(
            &request,
            r#"{"document":{"id":"123:456","name":"Root Frame"}}"#,
        )
        .expect("fixture payload should parse");

        let encoded = serde_json::to_string(&snapshot).expect("snapshot should serialize");
        let decoded: super::RawFigmaSnapshot =
            serde_json::from_str(&encoded).expect("snapshot should deserialize");

        assert_eq!(decoded, snapshot);
    }
}

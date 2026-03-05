pub mod normalizer;

use serde_json::{Value, json};

pub const RAW_SNAPSHOT_SCHEMA_VERSION: &str = "1.0";
pub const FIGMA_API_VERSION: &str = "v1";
pub const DEFAULT_FIGMA_API_BASE_URL: &str = "https://api.figma.com";

#[derive(Debug, thiserror::Error)]
pub enum FetchClientError {
    #[error("invalid fetch request: {0}")]
    InvalidRequest(String),
    #[error("invalid fixture json: {0}")]
    InvalidFixtureJson(#[from] serde_json::Error),
    #[error("figma api unauthorized")]
    Unauthorized,
    #[error("figma api returned non-success status {status}: {message}")]
    HttpStatus { status: u16, message: String },
    #[error("invalid figma api response: {0}")]
    InvalidApiResponse(String),
    #[error("http transport error: {0}")]
    HttpTransport(String),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFetchRequest {
    pub fetch: FetchNodesRequest,
    pub figma_token: String,
    pub api_base_url: Option<String>,
}

impl LiveFetchRequest {
    pub fn new(
        file_key: String,
        node_id: String,
        figma_token: String,
        api_base_url: Option<String>,
    ) -> Result<Self, FetchClientError> {
        let fetch = FetchNodesRequest::new(file_key, node_id)?;

        let figma_token = figma_token.trim().to_string();
        if figma_token.is_empty() {
            return Err(FetchClientError::InvalidRequest(
                "figma_token is required for live fetch".to_string(),
            ));
        }

        let api_base_url = api_base_url
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            fetch,
            figma_token,
            api_base_url,
        })
    }

    pub fn api_base_url(&self) -> &str {
        self.api_base_url
            .as_deref()
            .unwrap_or(DEFAULT_FIGMA_API_BASE_URL)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveScreenshotRequest {
    pub fetch: FetchNodesRequest,
    pub figma_token: String,
    pub api_base_url: Option<String>,
}

impl LiveScreenshotRequest {
    pub fn new(
        file_key: String,
        node_id: String,
        figma_token: String,
        api_base_url: Option<String>,
    ) -> Result<Self, FetchClientError> {
        let fetch = FetchNodesRequest::new(file_key, node_id)?;

        let figma_token = figma_token.trim().to_string();
        if figma_token.is_empty() {
            return Err(FetchClientError::InvalidRequest(
                "figma_token is required for screenshot fetch".to_string(),
            ));
        }

        let api_base_url = api_base_url
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            fetch,
            figma_token,
            api_base_url,
        })
    }

    pub fn api_base_url(&self) -> &str {
        self.api_base_url
            .as_deref()
            .unwrap_or(DEFAULT_FIGMA_API_BASE_URL)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeScreenshot {
    pub node_id: String,
    pub image_url: String,
    pub format: String,
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

pub fn fetch_snapshot_live(
    request: &LiveFetchRequest,
) -> Result<RawFigmaSnapshot, FetchClientError> {
    fetch_snapshot_live_with_base_url(
        &request.fetch,
        request.figma_token.as_str(),
        request.api_base_url(),
    )
}

pub fn fetch_node_screenshot_live(
    request: &LiveScreenshotRequest,
) -> Result<NodeScreenshot, FetchClientError> {
    fetch_node_screenshot_live_with_base_url(
        &request.fetch,
        request.figma_token.as_str(),
        request.api_base_url(),
    )
}

pub fn fetch_snapshot_live_with_base_url(
    request: &FetchNodesRequest,
    figma_token: &str,
    api_base_url: &str,
) -> Result<RawFigmaSnapshot, FetchClientError> {
    let figma_token = figma_token.trim();
    if figma_token.is_empty() {
        return Err(FetchClientError::InvalidRequest(
            "figma_token is required for live fetch".to_string(),
        ));
    }
    let api_base_url = api_base_url.trim();
    if api_base_url.is_empty() {
        return Err(FetchClientError::InvalidRequest(
            "api_base_url is required for live fetch".to_string(),
        ));
    }

    let api_url = format!(
        "{}/v1/files/{}/nodes",
        api_base_url.trim_end_matches('/'),
        request.file_key
    );

    let response = reqwest::blocking::Client::new()
        .get(api_url)
        .header("X-Figma-Token", figma_token)
        .query(&[("ids", request.node_id.as_str())])
        .send()
        .map_err(|err| FetchClientError::HttpTransport(err.to_string()))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(FetchClientError::Unauthorized);
    }
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "response body unavailable".to_string());
        return Err(FetchClientError::HttpStatus {
            status: status.as_u16(),
            message: body,
        });
    }

    let payload = response
        .json::<Value>()
        .map_err(|err| FetchClientError::InvalidApiResponse(err.to_string()))?;
    build_snapshot_from_live_nodes_payload(request, payload)
}

pub fn fetch_node_screenshot_live_with_base_url(
    request: &FetchNodesRequest,
    figma_token: &str,
    api_base_url: &str,
) -> Result<NodeScreenshot, FetchClientError> {
    let figma_token = figma_token.trim();
    if figma_token.is_empty() {
        return Err(FetchClientError::InvalidRequest(
            "figma_token is required for screenshot fetch".to_string(),
        ));
    }
    let api_base_url = api_base_url.trim();
    if api_base_url.is_empty() {
        return Err(FetchClientError::InvalidRequest(
            "api_base_url is required for screenshot fetch".to_string(),
        ));
    }

    let api_url = format!(
        "{}/v1/images/{}",
        api_base_url.trim_end_matches('/'),
        request.file_key
    );

    let response = reqwest::blocking::Client::new()
        .get(api_url)
        .header("X-Figma-Token", figma_token)
        .query(&[("ids", request.node_id.as_str()), ("format", "png")])
        .send()
        .map_err(|err| FetchClientError::HttpTransport(err.to_string()))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(FetchClientError::Unauthorized);
    }
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "response body unavailable".to_string());
        return Err(FetchClientError::HttpStatus {
            status: status.as_u16(),
            message: body,
        });
    }

    let payload = response
        .json::<Value>()
        .map_err(|err| FetchClientError::InvalidApiResponse(err.to_string()))?;
    build_node_screenshot_from_payload(request, payload)
}

fn build_snapshot_from_live_nodes_payload(
    request: &FetchNodesRequest,
    payload: Value,
) -> Result<RawFigmaSnapshot, FetchClientError> {
    let document = payload
        .get("nodes")
        .and_then(Value::as_object)
        .and_then(|nodes| nodes.get(request.node_id.as_str()))
        .and_then(Value::as_object)
        .and_then(|node| node.get("document"))
        .cloned()
        .ok_or_else(|| {
            FetchClientError::InvalidApiResponse(format!(
                "missing nodes.{}.document in figma response",
                request.node_id
            ))
        })?;

    Ok(RawFigmaSnapshot {
        snapshot_version: RAW_SNAPSHOT_SCHEMA_VERSION.to_string(),
        source: RawSnapshotSource {
            file_key: request.file_key.clone(),
            node_id: request.node_id.clone(),
            figma_api_version: FIGMA_API_VERSION.to_string(),
        },
        payload: json!({
            "document": document
        }),
    })
}

fn build_node_screenshot_from_payload(
    request: &FetchNodesRequest,
    payload: Value,
) -> Result<NodeScreenshot, FetchClientError> {
    let image_url = payload
        .get("images")
        .and_then(Value::as_object)
        .and_then(|images| images.get(request.node_id.as_str()))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            FetchClientError::InvalidApiResponse(format!(
                "missing images.{} in figma response",
                request.node_id
            ))
        })?;

    Ok(NodeScreenshot {
        node_id: request.node_id.clone(),
        image_url: image_url.to_string(),
        format: "png".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::io::{Read, Write};

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

    #[test]
    fn live_fetch_request_rejects_missing_figma_token() {
        let err = super::LiveFetchRequest::new(
            "abc123".to_string(),
            "123:456".to_string(),
            "".to_string(),
            None,
        )
        .expect_err("empty figma token should be rejected");

        assert_eq!(
            err.to_string(),
            "invalid fetch request: figma_token is required for live fetch"
        );
    }

    #[test]
    fn live_fetch_request_allows_explicit_api_base_url_override() {
        let request = super::LiveFetchRequest::new(
            "abc123".to_string(),
            "123:456".to_string(),
            "secret-token".to_string(),
            Some("http://127.0.0.1:9999".to_string()),
        )
        .expect("live fetch request should be valid");

        assert_eq!(request.fetch.file_key, "abc123");
        assert_eq!(request.fetch.node_id, "123:456");
        assert_eq!(request.figma_token, "secret-token");
        assert_eq!(
            request.api_base_url,
            Some("http://127.0.0.1:9999".to_string())
        );
    }

    #[test]
    fn live_fetch_request_uses_default_figma_api_base_url() {
        let request = super::LiveFetchRequest::new(
            "abc123".to_string(),
            "123:456".to_string(),
            "secret-token".to_string(),
            None,
        )
        .expect("live fetch request should be valid");

        assert_eq!(request.api_base_url(), super::DEFAULT_FIGMA_API_BASE_URL);
    }

    #[test]
    fn fetch_client_error_contract_includes_live_transport_variants() {
        let unauthorized = super::FetchClientError::Unauthorized;
        assert_eq!(unauthorized.to_string(), "figma api unauthorized");

        let http_status = super::FetchClientError::HttpStatus {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert_eq!(
            http_status.to_string(),
            "figma api returned non-success status 404: Not Found"
        );
    }

    #[test]
    fn build_snapshot_from_live_nodes_payload_extracts_requested_document() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");
        let payload = serde_json::json!({
            "nodes": {
                "123:456": {
                    "document": {
                        "id": "123:456",
                        "name": "Live Root"
                    }
                }
            }
        });

        let snapshot = super::build_snapshot_from_live_nodes_payload(&request, payload)
            .expect("valid payload");

        assert_eq!(snapshot.source.file_key, "abc123");
        assert_eq!(snapshot.source.node_id, "123:456");
        assert_eq!(
            snapshot.payload,
            serde_json::json!({
                "document": {
                    "id": "123:456",
                    "name": "Live Root"
                }
            })
        );
    }

    #[test]
    fn build_snapshot_from_live_nodes_payload_requires_document_for_node() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");
        let payload = serde_json::json!({
            "nodes": {
                "123:456": {}
            }
        });

        let err = super::build_snapshot_from_live_nodes_payload(&request, payload)
            .expect_err("payload without document should fail");
        assert_eq!(
            err.to_string(),
            "invalid figma api response: missing nodes.123:456.document in figma response"
        );
    }

    #[test]
    fn fetch_snapshot_live_rejects_missing_figma_token() {
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");
        let err = super::fetch_snapshot_live_with_base_url(&request, "", "http://127.0.0.1:9")
            .expect_err("empty token should fail");

        assert_eq!(
            err.to_string(),
            "invalid fetch request: figma_token is required for live fetch"
        );
    }

    #[test]
    fn fetch_snapshot_live_with_base_url_sends_auth_header_and_maps_success() {
        let (base_url, request_rx, server_thread) = match start_single_response_server(
            "200 OK",
            r#"{
                "nodes": {
                    "123:456": {
                        "document": {
                            "id": "123:456",
                            "name": "Live Root"
                        }
                    }
                }
            }"#,
        ) {
            Ok(server) => server,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("skipping live transport test: local socket bind not permitted");
                return;
            }
            Err(err) => panic!("mock server should bind: {err}"),
        };
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let snapshot =
            super::fetch_snapshot_live_with_base_url(&request, "secret-token", &base_url)
                .expect("live fetch should succeed");
        let raw_request = request_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("mock server should receive request");
        server_thread.join().expect("server thread should finish");

        let lower_request = raw_request.to_ascii_lowercase();
        assert!(raw_request.starts_with("GET /v1/files/abc123/nodes?ids=123%3A456 HTTP/1.1"));
        assert!(lower_request.contains("x-figma-token: secret-token"));
        assert_eq!(
            snapshot.payload,
            serde_json::json!({
                "document": {
                    "id": "123:456",
                    "name": "Live Root"
                }
            })
        );
    }

    #[test]
    fn fetch_snapshot_live_maps_unauthorized_status() {
        let (base_url, _request_rx, server_thread) =
            match start_single_response_server("401 Unauthorized", "Unauthorized") {
                Ok(server) => server,
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    eprintln!("skipping live transport test: local socket bind not permitted");
                    return;
                }
                Err(err) => panic!("mock server should bind: {err}"),
            };
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let err = super::fetch_snapshot_live_with_base_url(&request, "secret-token", &base_url)
            .expect_err("unauthorized response should fail");
        server_thread.join().expect("server thread should finish");

        assert_eq!(err.to_string(), "figma api unauthorized");
    }

    #[test]
    fn fetch_snapshot_live_maps_non_success_status_with_body() {
        let (base_url, _request_rx, server_thread) =
            match start_single_response_server("404 Not Found", "No file") {
                Ok(server) => server,
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    eprintln!("skipping live transport test: local socket bind not permitted");
                    return;
                }
                Err(err) => panic!("mock server should bind: {err}"),
            };
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let err = super::fetch_snapshot_live_with_base_url(&request, "secret-token", &base_url)
            .expect_err("404 response should fail");
        server_thread.join().expect("server thread should finish");

        assert_eq!(
            err.to_string(),
            "figma api returned non-success status 404: No file"
        );
    }

    #[test]
    fn fetch_node_screenshot_live_with_base_url_requests_images_endpoint() {
        let (base_url, request_rx, server_thread) = match start_single_response_server(
            "200 OK",
            r#"{
                "images": {
                    "123:456": "https://cdn.example.com/image.png"
                }
            }"#,
        ) {
            Ok(server) => server,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("skipping live transport test: local socket bind not permitted");
                return;
            }
            Err(err) => panic!("mock server should bind: {err}"),
        };
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let screenshot =
            super::fetch_node_screenshot_live_with_base_url(&request, "secret-token", &base_url)
                .expect("screenshot fetch should succeed");
        let raw_request = request_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("mock server should receive request");
        server_thread.join().expect("server thread should finish");

        let lower_request = raw_request.to_ascii_lowercase();
        assert!(raw_request.starts_with("GET /v1/images/abc123?ids=123%3A456&format=png HTTP/1.1"));
        assert!(lower_request.contains("x-figma-token: secret-token"));
        assert_eq!(screenshot.node_id, "123:456");
        assert_eq!(screenshot.image_url, "https://cdn.example.com/image.png");
        assert_eq!(screenshot.format, "png");
    }

    #[test]
    fn fetch_node_screenshot_live_with_base_url_reports_missing_image_ref() {
        let (base_url, _request_rx, server_thread) =
            match start_single_response_server("200 OK", r#"{"images":{}}"#) {
                Ok(server) => server,
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    eprintln!("skipping live transport test: local socket bind not permitted");
                    return;
                }
                Err(err) => panic!("mock server should bind: {err}"),
            };
        let request = super::FetchNodesRequest::new("abc123".to_string(), "123:456".to_string())
            .expect("request should be valid");

        let err =
            super::fetch_node_screenshot_live_with_base_url(&request, "secret-token", &base_url)
                .expect_err("missing image ref should fail");
        server_thread.join().expect("server thread should finish");

        assert_eq!(
            err.to_string(),
            "invalid figma api response: missing images.123:456 in figma response"
        );
    }

    fn start_single_response_server(
        status_line: &str,
        body: &str,
    ) -> Result<
        (
            String,
            std::sync::mpsc::Receiver<String>,
            std::thread::JoinHandle<()>,
        ),
        std::io::Error,
    > {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let address = listener
            .local_addr()
            .expect("mock server should expose local address");
        let (request_tx, request_rx) = std::sync::mpsc::channel::<String>();
        let status_line = status_line.to_string();
        let body = body.to_string();

        let server_thread = std::thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("mock server should accept one request");
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .expect("mock server should set read timeout");

            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("mock server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buffer[..bytes_read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request = String::from_utf8_lossy(&request_bytes).to_string();
            let _ = request_tx.send(request);

            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}",
                content_length = body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("mock server should write response");
            stream.flush().expect("mock server should flush response");
        });

        Ok((format!("http://{address}"), request_rx, server_thread))
    }
}

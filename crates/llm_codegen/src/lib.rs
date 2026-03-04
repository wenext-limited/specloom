#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

pub const DEFAULT_LLM_API_BASE_URL: &str = "https://api.openai.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiRequest {
    pub model: String,
    pub target: String,
    pub bundle_path: PathBuf,
    pub output_dir: PathBuf,
    pub api_key: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateUiResult {
    pub written_files: Vec<String>,
    pub run_record_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateUiError {
    #[error("model is required")]
    MissingModel,
    #[error("target is required")]
    MissingTarget,
    #[error("bundle_path is required")]
    MissingBundlePath,
    #[error("output_dir is required")]
    MissingOutputDir,
    #[error("api_key is required")]
    MissingApiKey,
    #[error("failed to read `{path}`: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write `{path}`: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
    #[error("http transport error: {0}")]
    HttpTransport(String),
    #[error("llm api returned non-success status {status}: {message}")]
    HttpStatus { status: u16, message: String },
    #[error("invalid llm response: {0}")]
    InvalidResponse(String),
    #[error("invalid generated file path `{0}`")]
    InvalidGeneratedPath(String),
    #[error("serialization error: {0}")]
    Serialization(serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedFile {
    path: String,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedFilesPayload {
    files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmRunRecord {
    model: String,
    target: String,
    prompt_hash: String,
    input_bundle_hash: String,
    generated_files: Vec<String>,
    created_at_unix_ms: u128,
}

pub fn generate_ui(request: &GenerateUiRequest) -> Result<GenerateUiResult, GenerateUiError> {
    validate_request(request)?;

    let bundle_artifact = std::fs::read(request.bundle_path.as_path()).map_err(|source| {
        GenerateUiError::ReadFile {
            path: request.bundle_path.display().to_string(),
            source,
        }
    })?;
    let bundle_text = String::from_utf8_lossy(bundle_artifact.as_slice()).to_string();
    let prompt = build_prompt(request.target.as_str(), bundle_text.as_str());

    let api_url = format!(
        "{}/v1/responses",
        request
            .api_base_url
            .as_deref()
            .unwrap_or(DEFAULT_LLM_API_BASE_URL)
            .trim_end_matches('/')
    );
    let response = reqwest::blocking::Client::new()
        .post(api_url)
        .header(
            "Authorization",
            format!("Bearer {}", request.api_key.trim()),
        )
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": request.model.trim(),
            "input": prompt,
        }))
        .send()
        .map_err(|err| GenerateUiError::HttpTransport(err.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "response body unavailable".to_string());
        return Err(GenerateUiError::HttpStatus {
            status: status.as_u16(),
            message: body,
        });
    }

    let response_json = response
        .json::<serde_json::Value>()
        .map_err(|err| GenerateUiError::HttpTransport(err.to_string()))?;
    let files = parse_generated_files(&response_json)?;
    let written_files = write_generated_files(request.output_dir.as_path(), files.as_slice())?;
    let run_record_path = write_run_record(
        request,
        prompt.as_str(),
        bundle_artifact.as_slice(),
        written_files.as_slice(),
    )?;

    Ok(GenerateUiResult {
        written_files,
        run_record_path,
    })
}

fn validate_request(request: &GenerateUiRequest) -> Result<(), GenerateUiError> {
    if request.model.trim().is_empty() {
        return Err(GenerateUiError::MissingModel);
    }
    if request.target.trim().is_empty() {
        return Err(GenerateUiError::MissingTarget);
    }
    if request.bundle_path.as_os_str().is_empty() {
        return Err(GenerateUiError::MissingBundlePath);
    }
    if request.output_dir.as_os_str().is_empty() {
        return Err(GenerateUiError::MissingOutputDir);
    }
    if request.api_key.trim().is_empty() {
        return Err(GenerateUiError::MissingApiKey);
    }
    Ok(())
}

fn build_prompt(target: &str, bundle_json: &str) -> String {
    format!(
        "You are generating UI code for target `{target}`.\n\
Return JSON only with this schema:\n\
{{\"files\":[{{\"path\":\"relative/path.ext\",\"content\":\"...\"}}]}}\n\
Do not include markdown fences.\n\
Use the following bundle JSON as input context:\n{bundle_json}\n"
    )
}

fn parse_generated_files(
    response_json: &serde_json::Value,
) -> Result<Vec<GeneratedFile>, GenerateUiError> {
    if let Some(files_value) = response_json.get("files") {
        let payload = serde_json::from_value::<GeneratedFilesPayload>(serde_json::json!({
            "files": files_value
        }))
        .map_err(GenerateUiError::Serialization)?;
        return Ok(payload.files);
    }

    if let Some(output_text) = response_json
        .get("output_text")
        .and_then(serde_json::Value::as_str)
    {
        let payload = serde_json::from_str::<GeneratedFilesPayload>(output_text)
            .map_err(GenerateUiError::Serialization)?;
        return Ok(payload.files);
    }

    let nested_output_text = response_json
        .get("output")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("content"))
        .and_then(serde_json::Value::as_array)
        .and_then(|contents| contents.first())
        .and_then(|content| content.get("text"))
        .and_then(serde_json::Value::as_str);
    if let Some(output_text) = nested_output_text {
        let payload = serde_json::from_str::<GeneratedFilesPayload>(output_text)
            .map_err(GenerateUiError::Serialization)?;
        return Ok(payload.files);
    }

    Err(GenerateUiError::InvalidResponse(
        "missing generated files payload".to_string(),
    ))
}

fn write_generated_files(
    output_dir: &Path,
    files: &[GeneratedFile],
) -> Result<Vec<String>, GenerateUiError> {
    std::fs::create_dir_all(output_dir).map_err(|source| GenerateUiError::WriteFile {
        path: output_dir.display().to_string(),
        source,
    })?;

    let mut written_files = Vec::new();
    for generated in files {
        let relative_path = Path::new(generated.path.as_str());
        if relative_path.is_absolute()
            || relative_path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(GenerateUiError::InvalidGeneratedPath(
                generated.path.clone(),
            ));
        }

        let output_path = output_dir.join(relative_path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| GenerateUiError::WriteFile {
                path: parent.display().to_string(),
                source,
            })?;
        }
        std::fs::write(&output_path, generated.content.as_bytes()).map_err(|source| {
            GenerateUiError::WriteFile {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        written_files.push(output_path.display().to_string());
    }

    Ok(written_files)
}

fn write_run_record(
    request: &GenerateUiRequest,
    prompt: &str,
    bundle_bytes: &[u8],
    written_files: &[String],
) -> Result<String, GenerateUiError> {
    let run_dir = request
        .bundle_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    std::fs::create_dir_all(run_dir.as_path()).map_err(|source| GenerateUiError::WriteFile {
        path: run_dir.display().to_string(),
        source,
    })?;

    let created_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis();
    let record_path = run_dir.join(format!("run-{created_at_unix_ms}.json"));
    let record = LlmRunRecord {
        model: request.model.trim().to_string(),
        target: request.target.trim().to_string(),
        prompt_hash: sha256_hex(prompt.as_bytes()),
        input_bundle_hash: sha256_hex(bundle_bytes),
        generated_files: written_files.to_vec(),
        created_at_unix_ms,
    };
    let mut encoded = serde_json::to_vec_pretty(&record).map_err(GenerateUiError::Serialization)?;
    encoded.push(b'\n');
    std::fs::write(&record_path, encoded).map_err(|source| GenerateUiError::WriteFile {
        path: record_path.display().to_string(),
        source,
    })?;

    Ok(record_path.display().to_string())
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
    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn generate_ui_rejects_missing_required_values() {
        let request = GenerateUiRequest {
            model: String::new(),
            target: "swiftui".to_string(),
            bundle_path: PathBuf::from("bundle.json"),
            output_dir: PathBuf::from("output"),
            api_key: "secret".to_string(),
            api_base_url: None,
        };
        assert!(matches!(
            generate_ui(&request),
            Err(GenerateUiError::MissingModel)
        ));

        let request = GenerateUiRequest {
            model: "gpt-5".to_string(),
            target: String::new(),
            bundle_path: PathBuf::from("bundle.json"),
            output_dir: PathBuf::from("output"),
            api_key: "secret".to_string(),
            api_base_url: None,
        };
        assert!(matches!(
            generate_ui(&request),
            Err(GenerateUiError::MissingTarget)
        ));

        let request = GenerateUiRequest {
            model: "gpt-5".to_string(),
            target: "swiftui".to_string(),
            bundle_path: PathBuf::from("bundle.json"),
            output_dir: PathBuf::from("output"),
            api_key: String::new(),
            api_base_url: None,
        };
        assert!(matches!(
            generate_ui(&request),
            Err(GenerateUiError::MissingApiKey)
        ));
    }

    #[test]
    fn generate_ui_writes_files_and_run_record_from_mock_response() {
        let workspace_root =
            unique_test_workspace_root("generate_ui_writes_files_and_run_record_from_mock");
        let bundle_dir = workspace_root.join("output/llm");
        std::fs::create_dir_all(&bundle_dir).unwrap();
        let bundle_path = bundle_dir.join("llm_bundle.json");
        std::fs::write(
            &bundle_path,
            "{\n  \"bundle_version\": \"1.0\",\n  \"target\": \"swiftui\"\n}\n",
        )
        .unwrap();
        let output_dir = workspace_root.join("generated-ui");

        let (base_url, request_rx, server_thread) = start_single_response_server(
            "200 OK",
            r#"{
                "files": [
                    { "path": "App.swift", "content": "import SwiftUI\n" },
                    { "path": "Components/Button.swift", "content": "struct ButtonView {}\n" }
                ]
            }"#,
        );
        let request = GenerateUiRequest {
            model: "gpt-5".to_string(),
            target: "swiftui".to_string(),
            bundle_path: bundle_path.clone(),
            output_dir: output_dir.clone(),
            api_key: "secret-token".to_string(),
            api_base_url: Some(base_url),
        };
        let result = generate_ui(&request).unwrap();
        let raw_request = request_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        server_thread.join().unwrap();

        let lower_request = raw_request.to_ascii_lowercase();
        assert!(raw_request.starts_with("POST /v1/responses HTTP/1.1"));
        assert!(lower_request.contains("authorization: bearer secret-token"));

        assert_eq!(result.written_files.len(), 2);
        assert!(output_dir.join("App.swift").is_file());
        assert!(output_dir.join("Components/Button.swift").is_file());
        assert!(Path::new(result.run_record_path.as_str()).is_file());

        let run_record = std::fs::read_to_string(result.run_record_path).unwrap();
        assert!(run_record.contains("\"model\": \"gpt-5\""));
        assert!(run_record.contains("\"target\": \"swiftui\""));

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    fn start_single_response_server(
        status_line: &str,
        body: &str,
    ) -> (
        String,
        std::sync::mpsc::Receiver<String>,
        std::thread::JoinHandle<()>,
    ) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = std::sync::mpsc::channel::<String>();
        let status_line = status_line.to_string();
        let body = body.to_string();

        let server_thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();

            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream.read(&mut buffer).unwrap();
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
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });

        (format!("http://{address}"), request_rx, server_thread)
    }

    fn unique_test_workspace_root(test_name: &str) -> std::path::PathBuf {
        let timestamp_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "forge-llm-codegen-{test_name}-{}-{timestamp_nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
        path
    }
}

use std::io::{Read, Write};

#[test]
fn help_lists_pipeline_subcommands() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("fetch"));
    assert!(text.contains("normalize"));
    assert!(text.contains("generate"));
    assert!(text.contains("Examples:"));
    assert!(text.contains("agent-tool find-nodes"));
}

#[test]
fn generate_subcommand_help_includes_stage_order() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("fetch -> normalize -> build-spec"));
    assert!(text.contains("build-agent-context -> export-assets"));
}

#[test]
fn fetch_subcommand_help_describes_input_flags() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["fetch", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Input mode: `fixture`, `live`, or `snapshot`"));
    assert!(text.contains("--snapshot-path"));
    assert!(text.contains("FIGMA_TOKEN"));
}

#[test]
fn agent_tool_get_node_screenshot_help_describes_requirements() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["agent-tool", "get-node-screenshot", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Fetch a single-node screenshot from Figma"));
    assert!(text.contains("--file-key"));
    assert!(text.contains("--node-id"));
    assert!(text.contains("--figma-token"));
}

#[test]
fn fetch_subcommand_prints_stage_output_directory() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("fetch")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("stage=fetch"));
    assert!(text.contains("output=output/raw"));
}

#[test]
fn fetch_subcommand_rejects_live_input_without_required_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["fetch", "--input", "live"])
        .env_remove("FIGMA_TOKEN")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("live input missing required value(s)"));
    assert!(stderr.contains("--file-key"));
    assert!(stderr.contains("--node-id"));
    assert!(stderr.contains("FIGMA_TOKEN (or --figma-token)"));
}

#[test]
fn fetch_subcommand_rejects_snapshot_input_without_required_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["fetch", "--input", "snapshot"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("snapshot input missing required value(s)"));
    assert!(stderr.contains("--snapshot-path"));
}

#[test]
fn fetch_subcommand_accepts_snapshot_input_with_snapshot_path() {
    let workspace_root =
        unique_cli_workspace_root("fetch_subcommand_accepts_snapshot_input_with_snapshot_path");
    let snapshot_path = workspace_root.join("fixtures/source_snapshot.json");
    std::fs::create_dir_all(snapshot_path.parent().unwrap()).unwrap();
    std::fs::write(
        snapshot_path.as_path(),
        r#"{
            "snapshot_version": "1.0",
            "source": {
                "file_key": "snapshot-file-key",
                "node_id": "7:7",
                "figma_api_version": "v1"
            },
            "payload": {
                "document": {
                    "id": "7:7",
                    "name": "Snapshot Root",
                    "type": "FRAME",
                    "children": []
                }
            }
        }"#,
    )
    .unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "fetch",
            "--input",
            "snapshot",
            "--snapshot-path",
            "fixtures/source_snapshot.json",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=fetch"));
    assert!(text.contains("output=output/raw"));
    let artifact_path = workspace_root.join("output/raw/fetch_snapshot.json");
    assert!(artifact_path.is_file());

    let artifact = std::fs::read_to_string(&artifact_path).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(&artifact).unwrap();
    assert_eq!(decoded["source"]["file_key"], "snapshot-file-key");
    assert_eq!(decoded["source"]["node_id"], "7:7");

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn fetch_subcommand_uses_figma_token_from_env_for_live_input() {
    let workspace_root =
        unique_cli_workspace_root("fetch_subcommand_uses_figma_token_from_env_for_live_input");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "fetch",
            "--input",
            "live",
            "--file-key",
            "abc123",
            "--node-id",
            "123:456",
            "--figma-api-base-url",
            "http://127.0.0.1:9",
        ])
        .env("FIGMA_TOKEN", "token-from-env")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("fetch client error:"));
    assert!(stderr.contains("For live fetch, verify"));
    assert!(!stderr.contains("live input missing required value(s)"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn fetch_subcommand_accepts_figma_quick_link_for_live_input() {
    let workspace_root =
        unique_cli_workspace_root("fetch_subcommand_accepts_figma_quick_link_for_live_input");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "fetch",
            "--input",
            "live",
            "--figma-url",
            "https://www.figma.com/design/iGk9NrpbnaoODjdoWc2P0g/Ludo?node-id=79-18523&m=dev",
            "--figma-api-base-url",
            "http://127.0.0.1:9",
        ])
        .env("FIGMA_TOKEN", "token-from-env")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("fetch client error:"));
    assert!(stderr.contains("For live fetch, verify"));
    assert!(!stderr.contains("live input missing required value(s)"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn fetch_subcommand_rejects_invalid_figma_quick_link() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args([
            "fetch",
            "--input",
            "live",
            "--figma-url",
            "https://example.com/not-figma?node-id=79-18523",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("invalid --figma-url"));
    assert!(stderr.contains("figma.com"));
}

#[test]
fn stages_subcommand_lists_all_stage_outputs() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("stages")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(text.contains("stage=fetch output=output/raw"));
    assert!(text.contains("stage=normalize output=output/normalized"));
    assert!(text.contains("stage=build-spec output=output/specs"));
    assert!(text.contains("stage=build-agent-context output=output/agent"));
    assert!(text.contains("stage=export-assets output=output/assets"));
}

#[test]
fn run_stage_subcommand_runs_selected_stage() {
    let fetch_output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();
    assert!(fetch_output.status.success());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "normalize"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=normalize output=output/normalized"));
}

#[test]
fn run_stage_subcommand_rejects_unknown_stage() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "not-a-stage"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("unknown stage"));
    assert!(stderr.contains("not-a-stage"));
    assert!(stderr.contains("Valid stages:"));
    assert!(stderr.contains("Run `cli stages`"));
}

#[test]
fn stages_subcommand_supports_json_output_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["stages", "--output", "json"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        text.trim(),
        r#"{"stages":[{"stage":"fetch","output":"output/raw"},{"stage":"normalize","output":"output/normalized"},{"stage":"build-spec","output":"output/specs"},{"stage":"build-agent-context","output":"output/agent"},{"stage":"export-assets","output":"output/assets"}]}"#
    );
}

#[test]
fn run_stage_subcommand_supports_json_output_mode() {
    let fetch_output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();
    assert!(fetch_output.status.success());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "normalize", "--output", "json"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        text.trim(),
        r#"{"stage":"normalize","output":"output/normalized"}"#
    );
}

#[test]
fn run_stage_subcommand_rejects_unknown_stage_in_json_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "not-a-stage", "--output", "json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("unknown stage"));
    assert!(stderr.contains("not-a-stage"));
    assert!(stderr.contains("Valid stages:"));
}

#[test]
fn run_stage_subcommand_reports_missing_input_artifact_actionably() {
    let workspace_root =
        unique_cli_workspace_root("run_stage_subcommand_reports_missing_input_artifact");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["run-stage", "normalize"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("missing input artifact"));
    assert!(stderr.contains("run-stage fetch"));
    assert!(stderr.contains("cli generate"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_runs_full_pipeline() {
    let workspace_root = unique_cli_workspace_root("generate_subcommand_runs_full_pipeline");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=fetch output=output/raw"));
    assert!(text.contains("stage=normalize output=output/normalized"));
    assert!(text.contains("stage=build-spec output=output/specs"));
    assert!(text.contains("stage=build-agent-context output=output/agent"));
    assert!(!text.contains("stage=gen-swiftui output=output/swift"));
    assert!(text.contains("stage=export-assets output=output/assets"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_rejects_live_input_without_required_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate", "--input", "live", "--file-key", "abc123"])
        .env_remove("FIGMA_TOKEN")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("live input missing required value(s)"));
    assert!(stderr.contains("--node-id"));
    assert!(stderr.contains("FIGMA_TOKEN (or --figma-token)"));
}

#[test]
fn generate_subcommand_defaults_to_live_and_rejects_missing_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("generate")
        .env_remove("FIGMA_TOKEN")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("live input missing required value(s)"));
    assert!(stderr.contains("--file-key (or --figma-url)"));
    assert!(stderr.contains("--node-id (or --figma-url)"));
    assert!(stderr.contains("FIGMA_TOKEN (or --figma-token)"));
}

#[test]
fn generate_subcommand_accepts_snapshot_input_with_snapshot_path() {
    let workspace_root =
        unique_cli_workspace_root("generate_subcommand_accepts_snapshot_input_with_snapshot_path");
    let snapshot_path = workspace_root.join("fixtures/source_snapshot.json");
    std::fs::create_dir_all(snapshot_path.parent().unwrap()).unwrap();
    std::fs::write(
        snapshot_path.as_path(),
        r#"{
            "snapshot_version": "1.0",
            "source": {
                "file_key": "snapshot-file-key",
                "node_id": "7:7",
                "figma_api_version": "v1"
            },
            "payload": {
                "document": {
                    "id": "7:7",
                    "name": "Snapshot Root",
                    "type": "FRAME",
                    "children": []
                }
            }
        }"#,
    )
    .unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "generate",
            "--input",
            "snapshot",
            "--snapshot-path",
            "fixtures/source_snapshot.json",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=fetch output=output/raw"));
    assert!(text.contains("stage=normalize output=output/normalized"));
    assert!(text.contains("stage=build-spec output=output/specs"));
    assert!(text.contains("stage=build-agent-context output=output/agent"));
    assert!(text.contains("stage=export-assets output=output/assets"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_live_downloads_root_screenshot() {
    let workspace_root =
        unique_cli_workspace_root("generate_subcommand_live_downloads_root_screenshot");
    let expected_png_bytes = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 1, 2, 3];

    let (image_base_url, image_server_thread) =
        match start_single_binary_response_server("/root.png", expected_png_bytes.as_slice()) {
            Ok(server) => server,
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("skipping live screenshot download test: local socket bind not permitted");
                return;
            }
            Err(err) => panic!("image server should bind: {err}"),
        };
    let image_url = format!("{image_base_url}/root.png");

    let (api_base_url, api_server_thread) = match start_live_api_server(image_url.as_str()) {
        Ok(server) => server,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            let _ = image_server_thread.join();
            eprintln!("skipping live screenshot download test: local socket bind not permitted");
            return;
        }
        Err(err) => panic!("api server should bind: {err}"),
    };

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "generate",
            "--input",
            "live",
            "--file-key",
            "abc123",
            "--node-id",
            "123:456",
            "--figma-token",
            "token-from-test",
            "--figma-api-base-url",
            api_base_url.as_str(),
        ])
        .output()
        .unwrap();

    api_server_thread.join().expect("api server should finish");
    image_server_thread.join().expect("image server should finish");

    assert!(
        output.status.success(),
        "generate should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let screenshot_path = workspace_root.join("output/images/root_123_456.png");
    assert!(screenshot_path.is_file(), "expected screenshot: {screenshot_path:?}");
    let screenshot_bytes = std::fs::read(screenshot_path).expect("screenshot should be readable");
    assert_eq!(screenshot_bytes, expected_png_bytes);

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_rejects_live_input_without_required_values_in_json_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate", "--input", "live", "--output", "json"])
        .env_remove("FIGMA_TOKEN")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        stderr.trim(),
        r#"{"error":"live input missing required value(s): --file-key (or --figma-url), --node-id (or --figma-url), FIGMA_TOKEN (or --figma-token). Provide the missing value(s) and retry."}"#
    );
}

#[test]
fn generate_subcommand_supports_json_output_mode() {
    let workspace_root = unique_cli_workspace_root("generate_subcommand_supports_json_output_mode");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture", "--output", "json"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        text.trim(),
        r#"{"results":[{"stage":"fetch","output":"output/raw","artifact":"output/raw/fetch_snapshot.json"},{"stage":"normalize","output":"output/normalized","artifact":"output/normalized/normalized_document.json"},{"stage":"build-spec","output":"output/specs","artifact":"output/specs/ui_spec.ron"},{"stage":"build-agent-context","output":"output/agent","artifact":"output/agent/agent_context.json"},{"stage":"export-assets","output":"output/assets","artifact":"output/assets/asset_manifest.json"}]}"#
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_returns_error_when_workspace_is_blocked() {
    let workspace_root =
        unique_cli_workspace_root("generate_subcommand_returns_error_when_workspace_is_blocked");
    std::fs::write(workspace_root.join("output"), "blocked").unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("io error"));
    assert!(stderr.contains("working directory is writable"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn agent_tool_find_nodes_json_mode_returns_candidates() {
    let workspace_root = unique_cli_workspace_root("agent_tool_find_nodes_json_mode");

    let generate = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args([
            "agent-tool",
            "find-nodes",
            "--query",
            "fixture root",
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":\"low_confidence\""));
    assert!(stdout.contains("\"node_id\":\"0:1\""));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn agent_tool_get_node_info_reports_not_found_actionably() {
    let workspace_root = unique_cli_workspace_root("agent_tool_get_node_info_not_found");

    let generate = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["agent-tool", "get-node-info", "--node-id", "missing"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "status=not_found node_id=missing\n"
    );
    assert!(output.stderr.is_empty());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "forge-cli-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

fn start_live_api_server(
    image_url: &str,
) -> Result<(String, std::thread::JoinHandle<()>), std::io::Error> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener
        .local_addr()
        .expect("api server should expose local address");
    listener
        .set_nonblocking(true)
        .expect("api server should set nonblocking");
    let image_url = image_url.to_string();

    let server_thread = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut served = 0usize;
        while served < 2 && std::time::Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(conn) => conn,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(err) => panic!("api server should accept request: {err}"),
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .expect("api server should set read timeout");

            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("api server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buffer[..bytes_read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request = String::from_utf8_lossy(&request_bytes);
            let first_line = request.lines().next().unwrap_or_default();

            let (status_line, body, content_type) =
                if first_line.starts_with("GET /v1/files/abc123/nodes?ids=123%3A456 HTTP/1.1") {
                    (
                        "200 OK",
                        r#"{
                            "nodes": {
                                "123:456": {
                                    "document": {
                                        "id": "123:456",
                                        "name": "Live Root",
                                        "type": "FRAME",
                                        "children": []
                                    }
                                }
                            }
                        }"#
                        .to_string(),
                        "application/json",
                    )
                } else if first_line
                    .starts_with("GET /v1/images/abc123?ids=123%3A456&format=png HTTP/1.1")
                {
                    (
                        "200 OK",
                        format!(r#"{{"images": {{"123:456": "{image_url}"}}}}"#),
                        "application/json",
                    )
                } else {
                    ("404 Not Found", "Not Found".to_string(), "text/plain")
                };

            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}",
                content_length = body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("api server should write response");
            stream.flush().expect("api server should flush response");
            served += 1;
        }
    });

    Ok((format!("http://{address}"), server_thread))
}

fn start_single_binary_response_server(
    expected_path: &str,
    body: &[u8],
) -> Result<(String, std::thread::JoinHandle<()>), std::io::Error> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener
        .local_addr()
        .expect("image server should expose local address");
    let expected_path = expected_path.to_string();
    let body = body.to_vec();

    let server_thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("image server should accept request");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .expect("image server should set read timeout");

        let mut request_bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let bytes_read = stream
                .read(&mut buffer)
                .expect("image server should read request bytes");
            if bytes_read == 0 {
                break;
            }
            request_bytes.extend_from_slice(&buffer[..bytes_read]);
            if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let request = String::from_utf8_lossy(&request_bytes);
        let first_line = request.lines().next().unwrap_or_default();
        let status_line = if first_line.starts_with(format!("GET {expected_path} ").as_str()) {
            "200 OK"
        } else {
            "404 Not Found"
        };
        let response_body = if status_line == "200 OK" {
            body.clone()
        } else {
            b"not found".to_vec()
        };

        let mut response_head = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: image/png\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n",
            content_length = response_body.len()
        )
        .into_bytes();
        response_head.extend_from_slice(response_body.as_slice());

        stream
            .write_all(response_head.as_slice())
            .expect("image server should write response");
        stream.flush().expect("image server should flush response");
    });

    Ok((format!("http://{address}"), server_thread))
}

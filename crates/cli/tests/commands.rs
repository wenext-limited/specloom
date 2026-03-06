mod support;

use support::{
    seed_bundle_instruction_sources, specloom_command, start_live_api_server,
    start_single_binary_response_server, unique_cli_workspace_root,
};

#[test]
fn help_lists_pipeline_subcommands() {
    let output = specloom_command().arg("--help").output().unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("fetch"));
    assert!(text.contains("normalize"));
    assert!(text.contains("generate"));
    assert!(text.contains("Examples:"));
    assert!(text.contains("agent-tool find-nodes"));
}

#[test]
fn generate_subcommand_help_includes_stage_order() {
    let output = specloom_command()
        .args(["generate", "--help"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("fetch -> normalize -> build-spec"));
    assert!(text.contains("build-agent-context -> export-assets"));
}

#[test]
fn fetch_subcommand_help_describes_input_flags() {
    let output = specloom_command()
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
    let output = specloom_command()
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
    let output = specloom_command().arg("fetch").output().unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("stage=fetch"));
    assert!(text.contains("output=output/raw"));
}

#[test]
fn fetch_subcommand_rejects_live_input_without_required_values() {
    let output = specloom_command()
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
fn generate_ui_subcommand_bootstraps_global_config_template_when_missing() {
    let workspace_root =
        unique_cli_workspace_root("generate_ui_subcommand_bootstraps_global_config_template");
    let home_root = unique_cli_workspace_root("specloom-config-home-bootstrap");

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .env("HOME", home_root.as_path())
        .args(["generate-ui", "--bundle", "missing.json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));

    let config_path = home_root.join(".config/specloom/config.toml");
    assert!(config_path.is_file());
    let config_contents = std::fs::read_to_string(config_path.as_path()).unwrap();
    assert!(config_contents.contains("[auth]"));
    assert!(config_contents.contains("# figma_token = \"...\""));
    assert!(config_contents.contains("# anthropic_api_key = \"...\""));

    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&home_root);
}

#[test]
fn fetch_subcommand_rejects_snapshot_input_without_required_values() {
    let output = specloom_command()
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

    let output = specloom_command()
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
    let output = specloom_command()
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
fn fetch_subcommand_uses_figma_token_from_global_config_for_live_input() {
    let workspace_root =
        unique_cli_workspace_root("fetch_subcommand_uses_figma_token_from_global_config");
    let home_root = unique_cli_workspace_root("specloom-config-home-for-fetch");
    seed_global_specloom_config(
        home_root.as_path(),
        r#"[auth]
figma_token = "token-from-config"
"#,
    );

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .env("HOME", home_root.as_path())
        .env_remove("FIGMA_TOKEN")
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
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("fetch client error:"));
    assert!(stderr.contains("For live fetch, verify"));
    assert!(!stderr.contains("live input missing required value(s)"));

    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&home_root);
}

#[test]
fn fetch_subcommand_accepts_figma_quick_link_for_live_input() {
    let workspace_root =
        unique_cli_workspace_root("fetch_subcommand_accepts_figma_quick_link_for_live_input");
    let output = specloom_command()
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
    let output = specloom_command()
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
    let output = specloom_command().arg("stages").output().unwrap();
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
    let fetch_output = specloom_command()
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();
    assert!(fetch_output.status.success());

    let output = specloom_command()
        .args(["run-stage", "normalize"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=normalize output=output/normalized"));
}

#[test]
fn run_stage_subcommand_rejects_unknown_stage() {
    let output = specloom_command()
        .args(["run-stage", "not-a-stage"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("unknown stage"));
    assert!(stderr.contains("not-a-stage"));
    assert!(stderr.contains("Valid stages:"));
    assert!(stderr.contains("Run `specloom stages`"));
}

#[test]
fn stages_subcommand_supports_json_output_mode() {
    let output = specloom_command()
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
    let fetch_output = specloom_command()
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();
    assert!(fetch_output.status.success());

    let output = specloom_command()
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
    let output = specloom_command()
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
    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["run-stage", "normalize"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("missing input artifact"));
    assert!(stderr.contains("run-stage fetch"));
    assert!(stderr.contains("specloom generate"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_runs_full_pipeline() {
    let workspace_root = unique_cli_workspace_root("generate_subcommand_runs_full_pipeline");

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("pipeline=generate"));
    assert!(text.contains("[1/5] RUN  stage=fetch"));
    assert!(text.contains("[5/5] DONE stage=export-assets"));
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
    let output = specloom_command()
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
    let output = specloom_command()
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

    let output = specloom_command()
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
    assert!(text.contains("pipeline=generate"));
    assert!(text.contains("[1/5] RUN  stage=fetch"));
    assert!(text.contains("[5/5] DONE stage=export-assets"));
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
                eprintln!(
                    "skipping live screenshot download test: local socket bind not permitted"
                );
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

    let output = specloom_command()
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
    image_server_thread
        .join()
        .expect("image server should finish");

    assert!(
        output.status.success(),
        "generate should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let screenshot_path = workspace_root.join("output/images/root_123_456.png");
    assert!(
        screenshot_path.is_file(),
        "expected screenshot: {screenshot_path:?}"
    );
    let screenshot_bytes = std::fs::read(screenshot_path).expect("screenshot should be readable");
    assert_eq!(screenshot_bytes, expected_png_bytes);

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_rejects_live_input_without_required_values_in_json_mode() {
    let output = specloom_command()
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

    let output = specloom_command()
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

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout.contains("pipeline=generate"));
    assert!(stdout.contains("[1/5] RUN  stage=fetch"));
    assert!(stdout.contains("[1/5] FAIL stage=fetch"));
    assert!(stderr.contains("io error"));
    assert!(stderr.contains("working directory is writable"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn prepare_llm_bundle_subcommand_writes_bundle_path() {
    let workspace_root =
        unique_cli_workspace_root("prepare_llm_bundle_subcommand_writes_bundle_path");

    seed_bundle_instruction_sources(workspace_root.as_path());

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .args([
            "prepare-llm-bundle",
            "--figma-url",
            "https://www.figma.com/design/abc/Login?node-id=1-2",
            "--target",
            "react-tailwind",
            "--intent",
            "Generate login screen code",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "prepare should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("stage=prepare-llm-bundle"));
    assert!(stdout.contains("artifact=output/agent/llm_bundle.json"));

    let bundle_path = workspace_root.join("output/agent/llm_bundle.json");
    assert!(bundle_path.is_file());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_ui_subcommand_reports_generated_artifact_paths() {
    let workspace_root =
        unique_cli_workspace_root("generate_ui_subcommand_reports_generated_artifact_paths");
    seed_bundle_instruction_sources(workspace_root.as_path());

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let prepare = specloom_command()
        .current_dir(workspace_root.as_path())
        .args([
            "prepare-llm-bundle",
            "--figma-url",
            "https://www.figma.com/design/abc/Login?node-id=1-2",
            "--target",
            "react-tailwind",
            "--intent",
            "Generate login screen code",
        ])
        .output()
        .unwrap();
    assert!(prepare.status.success());

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate-ui", "--bundle", "output/agent/llm_bundle.json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "generate-ui should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("stage=generate-ui"));
    assert!(stdout.contains("output/generated/"));
    assert!(
        workspace_root
            .join("output/generated/react-tailwind/App.tsx")
            .is_file()
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_ui_subcommand_rejects_anthropic_without_api_key() {
    let workspace_root =
        unique_cli_workspace_root("generate_ui_subcommand_rejects_anthropic_without_api_key");
    seed_bundle_instruction_sources(workspace_root.as_path());

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let prepare = specloom_command()
        .current_dir(workspace_root.as_path())
        .args([
            "prepare-llm-bundle",
            "--figma-url",
            "https://www.figma.com/design/abc/Login?node-id=1-2",
            "--target",
            "react-tailwind",
            "--intent",
            "Generate login screen code",
        ])
        .output()
        .unwrap();
    assert!(prepare.status.success());

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .env_remove("ANTHROPIC_API_KEY")
        .args([
            "generate-ui",
            "--bundle",
            "output/agent/llm_bundle.json",
            "--provider",
            "anthropic",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.trim().contains(
        "anthropic provider missing required value(s): ANTHROPIC_API_KEY (or --api-key)"
    ));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_ui_subcommand_uses_anthropic_api_key_from_global_config() {
    let workspace_root =
        unique_cli_workspace_root("generate_ui_subcommand_uses_anthropic_api_key_from_config");
    let home_root = unique_cli_workspace_root("specloom-config-home-for-anthropic");
    seed_global_specloom_config(
        home_root.as_path(),
        r#"[auth]
anthropic_api_key = "config-anthropic-key"
"#,
    );
    seed_bundle_instruction_sources(workspace_root.as_path());

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let prepare = specloom_command()
        .current_dir(workspace_root.as_path())
        .args([
            "prepare-llm-bundle",
            "--figma-url",
            "https://www.figma.com/design/abc/Login?node-id=1-2",
            "--target",
            "react-tailwind",
            "--intent",
            "Generate login screen code",
        ])
        .output()
        .unwrap();
    assert!(prepare.status.success());

    let output = specloom_command()
        .current_dir(workspace_root.as_path())
        .env("HOME", home_root.as_path())
        .env_remove("ANTHROPIC_API_KEY")
        .args([
            "generate-ui",
            "--bundle",
            "output/agent/llm_bundle.json",
            "--provider",
            "anthropic",
            "--model",
            "claude-3-5-sonnet-latest",
            "--api-base-url",
            "http://127.0.0.1:9",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("agent runner error:"));
    assert!(stderr.contains("anthropic generation failed"));
    assert!(!stderr.contains("anthropic provider missing required value(s)"));

    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&home_root);
}

#[test]
fn agent_tool_find_nodes_json_mode_returns_candidates() {
    let workspace_root = unique_cli_workspace_root("agent_tool_find_nodes_json_mode");

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let output = specloom_command()
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

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(generate.status.success());

    let output = specloom_command()
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

fn seed_global_specloom_config(home_root: &std::path::Path, contents: &str) {
    let config_path = home_root.join(".config/specloom/config.toml");
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).expect("config parent should be creatable");
    }
    std::fs::write(config_path.as_path(), contents).expect("config should be writable");
}

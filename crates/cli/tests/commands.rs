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
    assert!(text.contains("stage=infer-layout output=output/inferred"));
    assert!(text.contains("stage=build-spec output=output/specs"));
    assert!(text.contains("stage=gen-swiftui output=output/swift"));
    assert!(text.contains("stage=export-assets output=output/assets"));
    assert!(text.contains("stage=report output=output/reports"));
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
        r#"{"stages":[{"stage":"fetch","output":"output/raw"},{"stage":"normalize","output":"output/normalized"},{"stage":"infer-layout","output":"output/inferred"},{"stage":"build-spec","output":"output/specs"},{"stage":"gen-swiftui","output":"output/swift"},{"stage":"export-assets","output":"output/assets"},{"stage":"report","output":"output/reports"}]}"#
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
        .arg("generate")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(text.contains("stage=fetch output=output/raw"));
    assert!(text.contains("stage=normalize output=output/normalized"));
    assert!(text.contains("stage=infer-layout output=output/inferred"));
    assert!(text.contains("stage=build-spec output=output/specs"));
    assert!(text.contains("stage=gen-swiftui output=output/swift"));
    assert!(text.contains("stage=export-assets output=output/assets"));
    assert!(text.contains("stage=report output=output/reports"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_subcommand_rejects_live_input_without_required_values() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate", "--input", "live", "--file-key", "abc123"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("live input missing required value(s)"));
    assert!(stderr.contains("--node-id"));
    assert!(stderr.contains("FIGMA_TOKEN (or --figma-token)"));
}

#[test]
fn generate_subcommand_rejects_live_input_without_required_values_in_json_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate", "--input", "live", "--output", "json"])
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
        .args(["generate", "--output", "json"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert_eq!(
        text.trim(),
        r#"{"results":[{"stage":"fetch","output":"output/raw","artifact":"output/raw/fetch_snapshot.json"},{"stage":"normalize","output":"output/normalized","artifact":"output/normalized/normalized_document.json"},{"stage":"infer-layout","output":"output/inferred","artifact":"output/inferred/layout_inference.json"},{"stage":"build-spec","output":"output/specs","artifact":"output/specs/ui_spec.json"},{"stage":"gen-swiftui","output":"output/swift","artifact":"output/swift/FixtureRootView.swift"},{"stage":"export-assets","output":"output/assets","artifact":"output/assets/asset_manifest.json"},{"stage":"report","output":"output/reports","artifact":"output/reports/review_report.json"}]}"#
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
        .arg("generate")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("io error"));
    assert!(stderr.contains("working directory is writable"));

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

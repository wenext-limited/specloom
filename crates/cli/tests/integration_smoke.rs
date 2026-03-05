#[test]
fn cli_help_smoke() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(out.status.success());
}

#[test]
fn run_stage_success_smoke() {
    let fetch_out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();

    assert!(fetch_out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&fetch_out.stdout),
        "stage=fetch output=output/raw\n"
    );
    assert!(fetch_out.stderr.is_empty());

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["run-stage", "normalize"])
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "stage=normalize output=output/normalized\n"
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn run_stage_unknown_stage_smoke() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["run-stage", "not-a-stage"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown stage: not-a-stage"));
    assert!(stderr.contains("Valid stages:"));
}

#[test]
fn generate_defaults_to_live_and_requires_inputs_smoke() {
    let workspace_root = unique_cli_workspace_root("generate_defaults_to_live_and_requires_inputs");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(workspace_root.as_path())
        .arg("generate")
        .env_remove("FIGMA_TOKEN")
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("live input missing required value(s):"));
    assert!(stderr.contains("--file-key (or --figma-url)"));
    assert!(stderr.contains("--node-id (or --figma-url)"));
    assert!(stderr.contains("FIGMA_TOKEN (or --figma-token)"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_success_with_explicit_fixture_input_smoke() {
    let workspace_root =
        unique_cli_workspace_root("generate_success_with_explicit_fixture_input_smoke");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("pipeline=generate"));
    assert!(stdout.contains("stages=5"));
    assert!(stdout.contains("[1/5] RUN  stage=fetch"));
    assert!(stdout.contains("[5/5] DONE stage=export-assets"));
    assert!(
        stdout.contains("stage=fetch output=output/raw artifact=output/raw/fetch_snapshot.json")
    );
    assert!(stdout.contains("stage=normalize output=output/normalized artifact=output/normalized/normalized_document.json"));
    assert!(
        stdout.contains("stage=build-spec output=output/specs artifact=output/specs/ui_spec.ron")
    );
    assert!(stdout.contains(
        "stage=build-agent-context output=output/agent artifact=output/agent/agent_context.json"
    ));
    assert!(stdout.contains(
        "stage=export-assets output=output/assets artifact=output/assets/asset_manifest.json"
    ));
    assert!(out.stderr.is_empty());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "forge-cli-smoke-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

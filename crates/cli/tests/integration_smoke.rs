#[test]
fn cli_help_smoke() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(out.status.success());
}

#[test]
fn run_stage_success_smoke() {
    let fetch_out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["run-stage", "fetch"])
        .output()
        .unwrap();

    assert!(fetch_out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&fetch_out.stdout),
        "stage=fetch output=output/raw\n"
    );
    assert!(fetch_out.stderr.is_empty());

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
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
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
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
fn generate_success_smoke() {
    let workspace_root = unique_cli_workspace_root("generate_success_smoke");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .arg("generate")
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "stage=fetch output=output/raw artifact=output/raw/fetch_snapshot.json\nstage=normalize output=output/normalized artifact=output/normalized/normalized_document.json\nstage=infer-layout output=output/inferred artifact=output/inferred/layout_inference.json\nstage=build-spec output=output/specs artifact=output/specs/ui_spec.json\nstage=gen-swiftui output=output/swift artifact=output/swift/FixtureRootView.swift\nstage=export-assets output=output/assets artifact=output/assets/asset_manifest.json\nstage=report output=output/reports artifact=output/reports/review_report.json\n"
    );
    assert!(out.stderr.is_empty());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_success_with_explicit_fixture_input_smoke() {
    let workspace_root = unique_cli_workspace_root("generate_success_with_explicit_fixture_input_smoke");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "stage=fetch output=output/raw artifact=output/raw/fetch_snapshot.json\nstage=normalize output=output/normalized artifact=output/normalized/normalized_document.json\nstage=infer-layout output=output/inferred artifact=output/inferred/layout_inference.json\nstage=build-spec output=output/specs artifact=output/specs/ui_spec.json\nstage=gen-swiftui output=output/swift artifact=output/swift/FixtureRootView.swift\nstage=export-assets output=output/assets artifact=output/assets/asset_manifest.json\nstage=report output=output/reports artifact=output/reports/review_report.json\n"
    );
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

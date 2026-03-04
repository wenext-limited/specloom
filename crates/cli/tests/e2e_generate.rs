#[test]
fn generate_produces_all_expected_artifacts_from_fixture() {
    let workspace_root = unique_cli_workspace_root("generate_produces_all_expected_artifacts");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["generate", "--output", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "generate should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let expected_fixture = std::fs::read_to_string(format!(
        "{}/tests/fixtures/generate_expected_output.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("expected fixture should be readable");
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected_fixture).expect("expected fixture should be valid json");

    let actual_stdout = String::from_utf8_lossy(&output.stdout);
    let actual_json: serde_json::Value =
        serde_json::from_str(actual_stdout.as_ref()).expect("cli json output should be valid");
    assert_eq!(actual_json, expected_json);

    let expected_results = expected_json["results"]
        .as_array()
        .expect("fixture must contain results array");
    for result in expected_results {
        let artifact = result["artifact"]
            .as_str()
            .expect("each result must include artifact path");
        assert!(
            workspace_root.join(artifact).is_file(),
            "expected artifact to exist: {artifact}"
        );
    }

    let report_path = workspace_root.join("output/reports/review_report.json");
    let report_artifact =
        std::fs::read_to_string(&report_path).expect("report artifact should be readable");
    let report_json: serde_json::Value =
        serde_json::from_str(&report_artifact).expect("report artifact should be valid json");
    assert_eq!(report_json["summary"]["total_warnings"], 0);

    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "forge-cli-e2e-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

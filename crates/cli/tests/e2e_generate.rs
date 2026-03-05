#[test]
fn generate_produces_all_expected_artifacts_from_fixture() {
    let workspace_root = unique_cli_workspace_root("generate_produces_all_expected_artifacts");

    let expected_json = expected_generate_json_fixture();
    let actual_json = run_generate_json(workspace_root.as_path());
    assert_eq!(actual_json, expected_json);

    for artifact in expected_artifact_paths(&expected_json) {
        assert!(
            workspace_root.join(artifact.as_str()).is_file(),
            "expected artifact to exist: {artifact}",
        );
    }

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_outputs_are_byte_stable_across_reruns() {
    let workspace_root = unique_cli_workspace_root("generate_outputs_are_byte_stable");
    let expected_json = expected_generate_json_fixture();

    let first_json = run_generate_json(workspace_root.as_path());
    assert_eq!(first_json, expected_json);
    let first_artifacts = snapshot_artifacts(
        workspace_root.as_path(),
        expected_artifact_paths(&expected_json).as_slice(),
    );

    let second_json = run_generate_json(workspace_root.as_path());
    assert_eq!(second_json, expected_json);
    let second_artifacts = snapshot_artifacts(
        workspace_root.as_path(),
        expected_artifact_paths(&expected_json).as_slice(),
    );

    assert_eq!(first_artifacts, second_artifacts);

    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "specloom-cli-e2e-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

fn expected_generate_json_fixture() -> serde_json::Value {
    let expected_fixture = std::fs::read_to_string(format!(
        "{}/tests/fixtures/generate_expected_output.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("expected fixture should be readable");
    serde_json::from_str(&expected_fixture).expect("expected fixture should be valid json")
}

fn run_generate_json(workspace_root: &std::path::Path) -> serde_json::Value {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_specloom"))
        .current_dir(workspace_root)
        .args(["generate", "--input", "fixture", "--output", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "generate should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual_stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(actual_stdout.as_ref()).expect("cli json output should be valid")
}

fn expected_artifact_paths(expected_json: &serde_json::Value) -> Vec<String> {
    expected_json["results"]
        .as_array()
        .expect("fixture must contain results array")
        .iter()
        .map(|result| {
            result["artifact"]
                .as_str()
                .expect("each result must include artifact path")
                .to_string()
        })
        .collect::<Vec<_>>()
}

fn snapshot_artifacts(
    workspace_root: &std::path::Path,
    artifact_paths: &[String],
) -> std::collections::BTreeMap<String, Vec<u8>> {
    artifact_paths
        .iter()
        .map(|artifact| {
            let bytes = std::fs::read(workspace_root.join(artifact.as_str()))
                .expect("artifact should be readable");
            (artifact.clone(), bytes)
        })
        .collect()
}

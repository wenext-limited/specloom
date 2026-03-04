#[test]
fn help_lists_pipeline_subcommands() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("fetch"));
    assert!(text.contains("normalize"));
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
}

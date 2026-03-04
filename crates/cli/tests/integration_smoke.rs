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
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "unknown stage: not-a-stage\n"
    );
}

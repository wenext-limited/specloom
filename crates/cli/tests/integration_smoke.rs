#[test]
fn cli_help_smoke() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(out.status.success());
}

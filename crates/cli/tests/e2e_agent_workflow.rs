mod support;

use support::{seed_bundle_instruction_sources, specloom_command, unique_cli_workspace_root};

#[test]
fn fixture_agent_workflow_writes_bundle_generated_ui_and_reports() {
    let workspace_root = unique_cli_workspace_root("fixture_agent_workflow");
    seed_bundle_instruction_sources(workspace_root.as_path());

    let generate = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate", "--input", "fixture"])
        .output()
        .unwrap();
    assert!(
        generate.status.success(),
        "generate should succeed: {}",
        String::from_utf8_lossy(&generate.stderr)
    );

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
    assert!(
        prepare.status.success(),
        "prepare-llm-bundle should succeed: {}",
        String::from_utf8_lossy(&prepare.stderr)
    );

    let generate_ui = specloom_command()
        .current_dir(workspace_root.as_path())
        .args(["generate-ui", "--bundle", "output/agent/llm_bundle.json"])
        .output()
        .unwrap();
    assert!(
        generate_ui.status.success(),
        "generate-ui should succeed: {}",
        String::from_utf8_lossy(&generate_ui.stderr)
    );

    for artifact in [
        "output/agent/llm_bundle.json",
        "output/generated/react-tailwind/App.tsx",
        "output/reports/generation_warnings.json",
        "output/reports/generation_trace.json",
    ] {
        assert!(
            workspace_root.join(artifact).is_file(),
            "expected artifact to exist: {artifact}"
        );
    }

    let _ = std::fs::remove_dir_all(&workspace_root);
}

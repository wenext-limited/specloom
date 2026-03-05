use super::*;

#[test]
fn stages_are_reported_in_order() {
    assert_eq!(
        pipeline_stage_names(),
        vec![
            "fetch",
            "normalize",
            "build-spec",
            "build-agent-context",
            "export-assets"
        ]
    );
}

#[test]
fn stage_output_directories_are_reported() {
    assert_eq!(
        pipeline_stage_output_dirs(),
        vec![
            ("fetch", "output/raw"),
            ("normalize", "output/normalized"),
            ("build-spec", "output/specs"),
            ("build-agent-context", "output/agent"),
            ("export-assets", "output/assets"),
        ]
    );
}

#[test]
fn run_stage_returns_execution_result_for_known_stage() {
    let workspace_root =
        unique_test_workspace_root("run_stage_returns_execution_result_for_known_stage");
    let result =
        run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch stage should run");
    assert_eq!(
        result,
        StageExecutionResult {
            stage_name: "fetch",
            output_dir: "output/raw",
            artifact_path: Some("output/raw/fetch_snapshot.json".to_string()),
        }
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_unknown_stage_is_rejected() {
    let err = run_stage("not-a-stage").expect_err("unknown stage should fail");
    assert_eq!(err, PipelineError::UnknownStage("not-a-stage".to_string()));
    assert!(err.actionable_message().contains("Valid stages:"));
}

#[test]
fn run_stage_fetch_with_snapshot_input_writes_snapshot_artifact() {
    let workspace_root =
        unique_test_workspace_root("run_stage_fetch_with_snapshot_input_writes_snapshot_artifact");
    let input_snapshot_path = workspace_root.join("fixtures/snapshot.json");
    if let Some(parent) = input_snapshot_path.parent() {
        std::fs::create_dir_all(parent).expect("fixture parent should be creatable");
    }
    std::fs::write(
        input_snapshot_path.as_path(),
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
    .expect("fixture snapshot should be written");

    let config = PipelineRunConfig {
        fetch_mode: FetchMode::Snapshot(SnapshotFetchConfig {
            snapshot_path: "fixtures/snapshot.json".to_string(),
        }),
    };

    let result = run_stage_in_workspace_with_config("fetch", workspace_root.as_path(), &config)
        .expect("fetch stage should run");
    assert_eq!(result.stage_name, "fetch");
    assert_eq!(result.output_dir, "output/raw");

    let artifact_path = workspace_root.join("output/raw/fetch_snapshot.json");
    assert!(artifact_path.is_file());

    let artifact = std::fs::read_to_string(artifact_path).expect("snapshot should be readable");
    let snapshot: figma_client::RawFigmaSnapshot =
        serde_json::from_str(&artifact).expect("snapshot should decode");
    assert_eq!(snapshot.source.file_key, "snapshot-file-key");
    assert_eq!(snapshot.source.node_id, "7:7");

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_normalize_requires_raw_artifact() {
    let workspace_root = unique_test_workspace_root("run_stage_normalize_requires_raw_artifact");

    let err = run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect_err("normalize should fail when raw artifact is missing");

    assert_eq!(
        err,
        PipelineError::MissingInputArtifact("output/raw/fetch_snapshot.json".to_string())
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_build_spec_writes_ron_artifact() {
    let workspace_root = unique_test_workspace_root("run_stage_build_spec_writes_ron_artifact");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");

    let result = run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run");
    assert_eq!(
        result,
        StageExecutionResult {
            stage_name: "build-spec",
            output_dir: "output/specs",
            artifact_path: Some("output/specs/ui_spec.ron".to_string()),
        }
    );

    let artifact_path = workspace_root.join("output/specs/ui_spec.ron");
    assert!(artifact_path.is_file());
    let artifact = std::fs::read_to_string(artifact_path).expect("spec should be readable");
    assert!(artifact.contains("Container("));

    let pre_layout_path = workspace_root.join("output/specs/pre_layout.ron");
    assert!(pre_layout_path.is_file());
    let pre_layout =
        std::fs::read_to_string(pre_layout_path).expect("pre-layout should be readable");
    assert!(pre_layout.contains("Container("));

    let node_map_path = workspace_root.join("output/specs/node_map.json");
    assert!(node_map_path.is_file());
    let node_map = std::fs::read_to_string(node_map_path).expect("node map should be readable");
    let node_map_value: serde_json::Value =
        serde_json::from_str(node_map.as_str()).expect("node map should decode");
    assert_eq!(node_map_value["version"], "node_map/1.0");
    assert!(node_map_value["nodes"].is_object());

    let transform_plan_path = workspace_root.join("output/specs/transform_plan.json");
    assert!(transform_plan_path.is_file());
    let transform_plan =
        std::fs::read_to_string(transform_plan_path).expect("transform plan should be readable");
    let transform_plan_value: serde_json::Value =
        serde_json::from_str(transform_plan.as_str()).expect("transform plan should decode");
    assert_eq!(transform_plan_value["version"], "transform_plan/1.0");
    assert!(transform_plan_value["decisions"].is_array());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_export_assets_writes_asset_manifest_artifact() {
    let workspace_root =
        unique_test_workspace_root("run_stage_export_assets_writes_asset_manifest_artifact");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");

    let result = run_stage_in_workspace("export-assets", workspace_root.as_path())
        .expect("export-assets should run");
    assert_eq!(
        result,
        StageExecutionResult {
            stage_name: "export-assets",
            output_dir: "output/assets",
            artifact_path: Some("output/assets/asset_manifest.json".to_string()),
        }
    );

    let artifact_path = workspace_root.join("output/assets/asset_manifest.json");
    assert!(artifact_path.is_file());

    let artifact = std::fs::read_to_string(artifact_path).expect("manifest should be readable");
    let manifest: asset_pipeline::AssetManifest =
        serde_json::from_str(&artifact).expect("manifest should decode");
    assert_eq!(
        manifest.manifest_version,
        asset_pipeline::ASSET_MANIFEST_VERSION
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_build_agent_context_writes_agent_artifacts() {
    let workspace_root =
        unique_test_workspace_root("run_stage_build_agent_context_writes_agent_artifacts");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");

    let result = run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("build-agent-context should run");
    assert_eq!(
        result,
        StageExecutionResult {
            stage_name: "build-agent-context",
            output_dir: "output/agent",
            artifact_path: Some("output/agent/agent_context.json".to_string()),
        }
    );

    assert!(
        workspace_root
            .join("output/agent/agent_context.json")
            .is_file()
    );
    assert!(
        workspace_root
            .join("output/agent/search_index.json")
            .is_file()
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn prepare_llm_bundle_in_workspace_writes_bundle_artifact() {
    let workspace_root =
        unique_test_workspace_root("prepare_llm_bundle_in_workspace_writes_bundle_artifact");

    seed_full_fixture_pipeline(workspace_root.as_path());
    seed_bundle_instruction_sources(workspace_root.as_path());

    let result = prepare_llm_bundle_in_workspace(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
    )
    .expect("bundle should build");

    assert_eq!(result, "output/agent/llm_bundle.json");

    let bundle_path = workspace_root.join("output/agent/llm_bundle.json");
    assert!(bundle_path.is_file());

    let bundle_text = std::fs::read_to_string(bundle_path).expect("bundle should be readable");
    let bundle: LlmBundle =
        serde_json::from_str(bundle_text.as_str()).expect("bundle should decode");
    assert_eq!(bundle.version, LLM_BUNDLE_VERSION);
    assert_eq!(bundle.request.target, "react-tailwind");
    assert_eq!(
        bundle.request.intent,
        "Generate production-ready login screen"
    );
    assert_eq!(
        bundle.figma.source_url,
        "https://www.figma.com/design/abc/Login?node-id=1-2"
    );
    assert_eq!(bundle.figma.file_key, "fixture-file-key");
    assert_eq!(bundle.figma.root_node_id, "0:1");
    assert_eq!(bundle.artifacts.ui_spec.path, "output/specs/ui_spec.ron");
    assert!(!bundle.artifacts.ui_spec.sha256.is_empty());
    assert!(!bundle.instructions.skills_guide_markdown.is_empty());
    assert!(!bundle.instructions.agent_playbook_markdown.is_empty());
    assert!(!bundle.instructions.figma_ui_coder_markdown.is_empty());
    assert!(!bundle.instructions.skill_docs.is_empty());
    assert_eq!(
        bundle
            .tool_contract
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "find_nodes",
            "get_node_info",
            "get_node_screenshot",
            "get_asset",
        ]
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn prepare_llm_bundle_fetches_instructions_from_remote_release_when_local_files_are_missing() {
    let workspace_root = unique_test_workspace_root(
        "prepare_llm_bundle_fetches_instructions_from_remote_release_when_local_files_are_missing",
    );
    let config_root = unique_test_workspace_root(
        "prepare_llm_bundle_fetches_instructions_from_remote_release_when_local_files_are_missing-config-root",
    );
    seed_full_fixture_pipeline(workspace_root.as_path());

    let version = env!("CARGO_PKG_VERSION");
    let release_ref = format!("v{version}");
    let skills_markdown = r#"# Project Skills Guide

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use layout guidance.

2. `authoring-transform-plan`
Path: `.codex/skills/authoring-transform-plan/SKILL.md`
Use transform plan guidance.
"#;
    let routes = std::collections::BTreeMap::from([
        (
            format!("/{release_ref}/.codex/SKILLS.md"),
            skills_markdown.to_string(),
        ),
        (
            format!("/{release_ref}/docs/agent-playbook.md"),
            "# remote playbook".to_string(),
        ),
        (
            format!("/{release_ref}/docs/figma-ui-coder.md"),
            "# remote figma ui coder".to_string(),
        ),
        (
            format!("/{release_ref}/.codex/skills/recognizing-layout/SKILL.md"),
            "# remote recognizing-layout".to_string(),
        ),
        (
            format!("/{release_ref}/.codex/skills/authoring-transform-plan/SKILL.md"),
            "# remote authoring-transform-plan".to_string(),
        ),
    ]);
    let (base_url, request_rx, server_thread) =
        start_instruction_response_server(routes).expect("instruction server should start");

    let result = prepare_llm_bundle_in_workspace_with_instruction_overrides(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
        Some(base_url.as_str()),
        Some(config_root.as_path()),
    )
    .expect("bundle should build with remote instruction sources");

    assert_eq!(result, "output/agent/llm_bundle.json");

    let bundle_path = workspace_root.join("output/agent/llm_bundle.json");
    let bundle_text = std::fs::read_to_string(bundle_path).expect("bundle should be readable");
    let bundle: LlmBundle =
        serde_json::from_str(bundle_text.as_str()).expect("bundle should decode");
    assert!(
        bundle
            .instructions
            .skills_guide_markdown
            .contains("Active Skills")
    );
    assert_eq!(
        bundle.instructions.agent_playbook_markdown,
        "# remote playbook"
    );
    assert_eq!(
        bundle.instructions.figma_ui_coder_markdown,
        "# remote figma ui coder"
    );
    assert_eq!(bundle.instructions.skill_docs.len(), 2);
    assert!(bundle.instructions.skill_docs.iter().any(|doc| {
        doc.path == ".codex/skills/recognizing-layout/SKILL.md"
            && doc.markdown == "# remote recognizing-layout"
    }));
    assert!(bundle.instructions.skill_docs.iter().any(|doc| {
        doc.path == ".codex/skills/authoring-transform-plan/SKILL.md"
            && doc.markdown == "# remote authoring-transform-plan"
    }));

    let requests = collect_server_requests(request_rx);
    assert_eq!(requests.len(), 5);
    for request in requests {
        assert!(request.contains(&format!(" /{release_ref}/")));
    }

    server_thread
        .join()
        .expect("instruction server thread should join");
    let cache_root = config_root.join("skills_cache").join(release_ref);
    assert!(cache_root.join(".codex/SKILLS.md").is_file());
    assert!(cache_root.join("docs/agent-playbook.md").is_file());
    assert!(cache_root.join("docs/figma-ui-coder.md").is_file());
    assert!(
        cache_root
            .join(".codex/skills/recognizing-layout/SKILL.md")
            .is_file()
    );
    assert!(
        cache_root
            .join(".codex/skills/authoring-transform-plan/SKILL.md")
            .is_file()
    );
    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&config_root);
}

#[test]
fn prepare_llm_bundle_prefers_local_instruction_files_when_available() {
    let workspace_root = unique_test_workspace_root(
        "prepare_llm_bundle_prefers_local_instruction_files_when_available",
    );
    seed_full_fixture_pipeline(workspace_root.as_path());
    seed_bundle_instruction_sources(workspace_root.as_path());

    let result = prepare_llm_bundle_in_workspace_with_instruction_overrides(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
        Some("http://127.0.0.1:9"),
        None,
    )
    .expect("bundle should build from local instruction sources");
    assert_eq!(result, "output/agent/llm_bundle.json");

    let bundle_path = workspace_root.join("output/agent/llm_bundle.json");
    let bundle_text = std::fs::read_to_string(bundle_path).expect("bundle should be readable");
    let bundle: LlmBundle =
        serde_json::from_str(bundle_text.as_str()).expect("bundle should decode");
    assert!(
        bundle
            .instructions
            .skills_guide_markdown
            .contains("Active Skills")
    );
    assert_eq!(
        bundle.instructions.agent_playbook_markdown,
        "# agent playbook"
    );
    assert_eq!(
        bundle.instructions.figma_ui_coder_markdown,
        "# figma ui coder"
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn prepare_llm_bundle_reads_instruction_files_from_global_cache_when_present() {
    let workspace_root = unique_test_workspace_root(
        "prepare_llm_bundle_reads_instruction_files_from_global_cache_when_present",
    );
    let config_root = unique_test_workspace_root(
        "prepare_llm_bundle_reads_instruction_files_from_global_cache_when_present-config-root",
    );
    seed_full_fixture_pipeline(workspace_root.as_path());

    let version = env!("CARGO_PKG_VERSION");
    let release_ref = format!("v{version}");
    seed_cached_bundle_instruction_sources(config_root.as_path(), release_ref.as_str());

    let result = prepare_llm_bundle_in_workspace_with_instruction_overrides(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
        Some("http://127.0.0.1:9"),
        Some(config_root.as_path()),
    )
    .expect("bundle should build from cached instruction sources");
    assert_eq!(result, "output/agent/llm_bundle.json");

    let bundle_path = workspace_root.join("output/agent/llm_bundle.json");
    let bundle_text = std::fs::read_to_string(bundle_path).expect("bundle should be readable");
    let bundle: LlmBundle =
        serde_json::from_str(bundle_text.as_str()).expect("bundle should decode");
    assert_eq!(
        bundle.instructions.agent_playbook_markdown,
        "# cached agent playbook"
    );
    assert_eq!(
        bundle.instructions.figma_ui_coder_markdown,
        "# cached figma ui coder"
    );
    assert!(bundle.instructions.skill_docs.iter().any(|doc| {
        doc.path == ".codex/skills/recognizing-layout/SKILL.md"
            && doc.markdown == "# cached recognizing-layout"
    }));

    let _ = std::fs::remove_dir_all(&workspace_root);
    let _ = std::fs::remove_dir_all(&config_root);
}

#[test]
fn generate_ui_with_mock_runner_writes_generated_output() {
    let workspace_root =
        unique_test_workspace_root("generate_ui_with_mock_runner_writes_generated_output");

    seed_full_fixture_pipeline(workspace_root.as_path());
    seed_bundle_instruction_sources(workspace_root.as_path());
    prepare_llm_bundle_in_workspace(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
    )
    .expect("bundle should build");

    let result = generate_ui_in_workspace(
        workspace_root.as_path(),
        &GenerateUiRequest {
            bundle_path: "output/agent/llm_bundle.json".to_string(),
            provider: GenerateUiProvider::Mock,
            model: None,
            api_key: None,
            api_base_url: None,
        },
        &MockAgentRunner,
    )
    .expect("generate ui should succeed");

    assert!(
        result
            .generated_paths
            .iter()
            .any(|path| path.starts_with("output/generated/"))
    );
    assert!(
        workspace_root
            .join("output/generated/react-tailwind/App.tsx")
            .is_file()
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn generate_ui_in_workspace_always_emits_warning_and_trace_artifacts() {
    let workspace_root = unique_test_workspace_root(
        "generate_ui_in_workspace_always_emits_warning_and_trace_artifacts",
    );

    seed_full_fixture_pipeline(workspace_root.as_path());
    seed_bundle_instruction_sources(workspace_root.as_path());
    prepare_llm_bundle_in_workspace(
        workspace_root.as_path(),
        &PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Login?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
    )
    .expect("bundle should build");

    generate_ui_in_workspace(
        workspace_root.as_path(),
        &GenerateUiRequest {
            bundle_path: "output/agent/llm_bundle.json".to_string(),
            provider: GenerateUiProvider::Mock,
            model: None,
            api_key: None,
            api_base_url: None,
        },
        &MockAgentRunner,
    )
    .expect("generation should succeed");

    assert!(
        workspace_root
            .join("output/reports/generation_warnings.json")
            .is_file()
    );
    assert!(
        workspace_root
            .join("output/reports/generation_trace.json")
            .is_file()
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn build_agent_context_uses_transformed_final_spec() {
    let workspace_root =
        unique_test_workspace_root("build_agent_context_uses_transformed_final_spec");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");

    let seeded_plan = ui_spec::TransformPlan {
        version: ui_spec::TRANSFORM_PLAN_VERSION.to_string(),
        decisions: vec![ui_spec::TransformDecision {
            node_id: "0:1".to_string(),
            suggested_type: ui_spec::SuggestedNodeType::HStack,
            child_policy: ui_spec::ChildPolicy {
                mode: ui_spec::ChildPolicyMode::Keep,
                children: Vec::new(),
            },
            repeat_element_ids: None,
            confidence: 0.93,
            reason: "Agent recognized horizontal root".to_string(),
        }],
    };
    let seeded_plan_path = workspace_root.join(TRANSFORM_PLAN_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = seeded_plan_path.parent() {
        std::fs::create_dir_all(parent).expect("seeded plan parent should be creatable");
    }
    let seeded_plan_bytes =
        serde_json::to_vec_pretty(&seeded_plan).expect("seeded plan should serialize");
    std::fs::write(seeded_plan_path.as_path(), seeded_plan_bytes)
        .expect("seeded plan should be written");

    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");
    run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("build-agent-context should run fourth");

    let spec_ron = std::fs::read_to_string(workspace_root.join("output/specs/ui_spec.ron"))
        .expect("final spec should be readable");
    assert!(spec_ron.contains("HStack("));

    let search_index_json =
        std::fs::read_to_string(workspace_root.join("output/agent/search_index.json"))
            .expect("search index should be readable");
    let search_index: agent_context::SearchIndex =
        serde_json::from_str(search_index_json.as_str()).expect("search index should decode");
    let root_entry = search_index
        .entries
        .iter()
        .find(|entry| entry.node_id == "0:1")
        .expect("root entry should exist");
    assert_eq!(root_entry.node_type, "HSTACK");

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn build_agent_context_accepts_repeat_element_ids_as_node_metadata() {
    let workspace_root = unique_test_workspace_root(
        "build_agent_context_accepts_repeat_element_ids_as_node_metadata",
    );

    let spec = ui_spec::UiSpec::Container {
        id: "0:1".to_string(),
        name: "Root".to_string(),
        text: String::new(),
        children: vec![ui_spec::UiSpec::Text {
            id: "0:2".to_string(),
            name: "Row".to_string(),
            children: Vec::new(),
            repeat_element_ids: vec!["row-instance-1".to_string()],
        }],
        repeat_element_ids: vec!["root-instance-1".to_string()],
    };

    let spec_path = workspace_root.join(SPEC_ARTIFACT_RELATIVE_PATH);
    if let Some(parent) = spec_path.parent() {
        std::fs::create_dir_all(parent).expect("spec parent should be creatable");
    }
    let spec_ron = spec.to_pretty_ron().expect("spec should serialize");
    std::fs::write(spec_path.as_path(), spec_ron).expect("spec should be writable");

    run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("repeat metadata should not fail build-agent-context");
    assert!(
        workspace_root
            .join("output/agent/agent_context.json")
            .is_file()
    );
    assert!(
        workspace_root
            .join("output/agent/search_index.json")
            .is_file()
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn build_agent_context_reuses_cached_root_screenshot_when_present() {
    let workspace_root = unique_test_workspace_root(
        "build_agent_context_reuses_cached_root_screenshot_when_present",
    );

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");

    let cached_screenshot_path = workspace_root.join("output/images/root_0_1.png");
    if let Some(parent) = cached_screenshot_path.parent() {
        std::fs::create_dir_all(parent).expect("cached screenshot parent should be creatable");
    }
    let cached_bytes = vec![137, 80, 78, 71, 0, 1, 2, 3];
    std::fs::write(cached_screenshot_path.as_path(), cached_bytes.as_slice())
        .expect("cached screenshot should be writable");

    let live_config = PipelineRunConfig {
        fetch_mode: FetchMode::Live(LiveFetchConfig {
            file_key: "abc123".to_string(),
            node_id: "0:1".to_string(),
            figma_token: "token-from-test".to_string(),
            api_base_url: Some("http://127.0.0.1:9".to_string()),
        }),
    };

    let result = run_stage_in_workspace_with_config(
        "build-agent-context",
        workspace_root.as_path(),
        &live_config,
    )
    .expect("build-agent-context should reuse cached screenshot");
    assert_eq!(result.stage_name, "build-agent-context");
    assert!(
        workspace_root
            .join("output/agent/agent_context.json")
            .is_file()
    );

    let actual_bytes = std::fs::read(cached_screenshot_path.as_path())
        .expect("cached screenshot should remain readable");
    assert_eq!(actual_bytes, cached_bytes);

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn find_nodes_in_workspace_returns_ranked_candidates() {
    let workspace_root =
        unique_test_workspace_root("find_nodes_in_workspace_returns_ranked_candidates");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");
    run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("build-agent-context should run fourth");

    let result = find_nodes_in_workspace(workspace_root.as_path(), "fixture root", 5)
        .expect("find_nodes should succeed");
    assert_eq!(result.status, FindNodesStatus::LowConfidence);
    assert!(!result.candidates.is_empty());
    assert_eq!(result.candidates[0].node_id, "0:1");

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn get_node_info_in_workspace_returns_not_found_for_missing_node() {
    let workspace_root = unique_test_workspace_root("get_node_info_in_workspace_returns_not_found");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");
    run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("build-agent-context should run fourth");

    let result = get_node_info_in_workspace(workspace_root.as_path(), "missing")
        .expect("node info lookup should succeed");
    assert_eq!(result.status, NodeInfoStatus::NotFound);
    assert!(result.node.is_none());

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn tool_lookup_no_match_emits_warning_artifact_entry() {
    let workspace_root =
        unique_test_workspace_root("tool_lookup_no_match_emits_warning_artifact_entry");

    run_stage_in_workspace("fetch", workspace_root.as_path()).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root.as_path())
        .expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root.as_path())
        .expect("build-spec should run third");
    run_stage_in_workspace("build-agent-context", workspace_root.as_path())
        .expect("build-agent-context should run fourth");

    let result = find_nodes_in_workspace(workspace_root.as_path(), "query-that-does-not-match", 5)
        .expect("find_nodes should succeed");
    assert_eq!(result.status, FindNodesStatus::NoMatch);

    let warnings_path = workspace_root.join("output/reports/generation_warnings.json");
    assert!(warnings_path.is_file());

    let warnings_json =
        std::fs::read_to_string(warnings_path).expect("warnings artifact should be readable");
    let warnings: agent_context::GenerationWarnings =
        serde_json::from_str(warnings_json.as_str()).expect("warnings artifact should decode");
    assert!(
        warnings
            .warnings
            .iter()
            .any(|warning| warning.warning_type == "NODE_NOT_FOUND")
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_all_in_workspace_executes_stages_in_order() {
    let workspace_root =
        unique_test_workspace_root("run_all_in_workspace_executes_stages_in_order");

    let results = run_all_in_workspace(workspace_root.as_path()).expect("run-all should succeed");

    assert_eq!(
        results
            .iter()
            .map(|result| result.stage_name)
            .collect::<Vec<_>>(),
        vec![
            "fetch",
            "normalize",
            "build-spec",
            "build-agent-context",
            "export-assets"
        ]
    );
    assert_eq!(
        results
            .last()
            .and_then(|result| result.artifact_path.clone()),
        Some("output/assets/asset_manifest.json".to_string())
    );

    let _ = std::fs::remove_dir_all(&workspace_root);
}

#[test]
fn run_stage_fetch_with_live_config_surfaces_actionable_fetch_error() {
    let workspace_root =
        unique_test_workspace_root("run_stage_fetch_with_live_config_surfaces_fetch_error");

    let config = PipelineRunConfig {
        fetch_mode: FetchMode::Live(LiveFetchConfig {
            file_key: "abc123".to_string(),
            node_id: "123:456".to_string(),
            figma_token: "token-from-test".to_string(),
            api_base_url: Some("http://127.0.0.1:9".to_string()),
        }),
    };

    let err = run_stage_in_workspace_with_config("fetch", workspace_root.as_path(), &config)
        .expect_err("live fetch should fail for unreachable endpoint");
    let message = err.actionable_message();
    assert!(message.contains("fetch client error:"));
    assert!(message.contains("For live fetch, verify"));

    let _ = std::fs::remove_dir_all(&workspace_root);
}

fn unique_test_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "specloom-core-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace root should be created");
    path
}

fn collect_server_requests(request_rx: std::sync::mpsc::Receiver<String>) -> Vec<String> {
    let mut requests = Vec::new();
    loop {
        match request_rx.recv_timeout(std::time::Duration::from_millis(250)) {
            Ok(request) => requests.push(request),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    requests
}

fn start_instruction_response_server(
    routes: std::collections::BTreeMap<String, String>,
) -> Result<
    (
        String,
        std::sync::mpsc::Receiver<String>,
        std::thread::JoinHandle<()>,
    ),
    std::io::Error,
> {
    use std::io::{Read, Write};

    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    listener
        .set_nonblocking(true)
        .expect("server should support nonblocking");
    let address = listener
        .local_addr()
        .expect("server should expose local address");
    let expected_requests = routes.len();
    let (request_tx, request_rx) = std::sync::mpsc::channel::<String>();

    let server_thread = std::thread::spawn(move || {
        let mut served_requests = 0usize;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while served_requests < expected_requests && std::time::Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(value) => value,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            };
            served_requests += 1;

            stream
                .set_nonblocking(false)
                .expect("server stream should be blocking");
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .expect("server should set read timeout");
            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buffer[..bytes_read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request = String::from_utf8_lossy(&request_bytes).to_string();
            let _ = request_tx.send(request.clone());
            let request_path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let (status_line, body) = if let Some(body) = routes.get(request_path) {
                ("200 OK", body.clone())
            } else {
                ("404 Not Found", "{\"error\":\"not found\"}".to_string())
            };
            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("server should write response");
            stream.flush().expect("server should flush response");
        }
    });

    Ok((format!("http://{address}"), request_rx, server_thread))
}

fn seed_full_fixture_pipeline(workspace_root: &std::path::Path) {
    run_stage_in_workspace("fetch", workspace_root).expect("fetch should run first");
    run_stage_in_workspace("normalize", workspace_root).expect("normalize should run second");
    run_stage_in_workspace("build-spec", workspace_root).expect("build-spec should run third");
    run_stage_in_workspace("build-agent-context", workspace_root)
        .expect("build-agent-context should run fourth");
    run_stage_in_workspace("export-assets", workspace_root)
        .expect("export-assets should run fifth");
}

fn seed_bundle_instruction_sources(workspace_root: &std::path::Path) {
    let skills_guide_path = workspace_root.join(".codex/SKILLS.md");
    if let Some(parent) = skills_guide_path.parent() {
        std::fs::create_dir_all(parent).expect("skills guide parent should be creatable");
    }
    std::fs::write(
        skills_guide_path.as_path(),
        r#"# Project Skills Guide

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use layout guidance.

2. `authoring-transform-plan`
Path: `.codex/skills/authoring-transform-plan/SKILL.md`
Use transform plan guidance.

## Usage Order
1. Example only
"#,
    )
    .expect("skills guide should be writable");

    let recognizing_layout_path = workspace_root.join(".codex/skills/recognizing-layout/SKILL.md");
    if let Some(parent) = recognizing_layout_path.parent() {
        std::fs::create_dir_all(parent).expect("recognizing-layout parent should be creatable");
    }
    std::fs::write(recognizing_layout_path.as_path(), "# recognizing layout")
        .expect("recognizing-layout skill should be writable");

    let authoring_transform_path =
        workspace_root.join(".codex/skills/authoring-transform-plan/SKILL.md");
    if let Some(parent) = authoring_transform_path.parent() {
        std::fs::create_dir_all(parent)
            .expect("authoring-transform-plan parent should be creatable");
    }
    std::fs::write(
        authoring_transform_path.as_path(),
        "# authoring transform plan",
    )
    .expect("authoring-transform-plan skill should be writable");

    let playbook_path = workspace_root.join("docs/agent-playbook.md");
    if let Some(parent) = playbook_path.parent() {
        std::fs::create_dir_all(parent).expect("playbook parent should be creatable");
    }
    std::fs::write(playbook_path.as_path(), "# agent playbook")
        .expect("agent playbook should be writable");

    let figma_ui_coder_path = workspace_root.join("docs/figma-ui-coder.md");
    std::fs::write(figma_ui_coder_path.as_path(), "# figma ui coder")
        .expect("figma ui coder doc should be writable");
}

fn seed_cached_bundle_instruction_sources(config_root: &std::path::Path, release_ref: &str) {
    let cache_root = config_root.join("skills_cache").join(release_ref);
    let skills_guide_path = cache_root.join(".codex/SKILLS.md");
    if let Some(parent) = skills_guide_path.parent() {
        std::fs::create_dir_all(parent).expect("cached skills guide parent should be creatable");
    }
    std::fs::write(
        skills_guide_path.as_path(),
        r#"# Project Skills Guide

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use layout guidance.
"#,
    )
    .expect("cached skills guide should be writable");

    let recognizing_layout_path = cache_root.join(".codex/skills/recognizing-layout/SKILL.md");
    if let Some(parent) = recognizing_layout_path.parent() {
        std::fs::create_dir_all(parent).expect("cached skill parent should be creatable");
    }
    std::fs::write(
        recognizing_layout_path.as_path(),
        "# cached recognizing-layout",
    )
    .expect("cached skill should be writable");

    let playbook_path = cache_root.join("docs/agent-playbook.md");
    if let Some(parent) = playbook_path.parent() {
        std::fs::create_dir_all(parent).expect("cached playbook parent should be creatable");
    }
    std::fs::write(playbook_path.as_path(), "# cached agent playbook")
        .expect("cached playbook should be writable");

    let figma_ui_coder_path = cache_root.join("docs/figma-ui-coder.md");
    std::fs::write(figma_ui_coder_path.as_path(), "# cached figma ui coder")
        .expect("cached figma ui coder should be writable");
}

# End-to-End Agent Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a reproducible end-to-end agent workflow where developers prepare a bundle from a Figma URL and then generate target UI code with one follow-up command.

**Architecture:** Keep deterministic pipeline stages authoritative, then add a bundle boundary (`prepare-llm-bundle`) that snapshots artifacts plus instruction context (skills and playbooks). Execute generation through a separate `generate-ui` command that consumes only the bundle and emits generated code + warnings + trace. Introduce a mockable agent runner boundary so tests stay deterministic while provider integrations can evolve.

**Tech Stack:** Rust 2024 workspace, `clap`, `serde`, `serde_json`, existing `specloom-core` stage/runtime modules, existing CLI command integration tests.

---

Use `@test-driven-development` for each task, `@systematic-debugging` for unexpected command output, and `@verification-before-completion` before final merge.

### Task 1: Add LLM Bundle Contract Types in Core

**Files:**
- Create: `crates/core/src/llm_bundle.rs`
- Modify: `crates/core/src/lib.rs`
- Test: `crates/core/src/llm_bundle.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn llm_bundle_round_trip_json() {
    let bundle = LlmBundle::sample();
    let bytes = serde_json::to_vec_pretty(&bundle).expect("bundle should encode");
    let decoded: LlmBundle = serde_json::from_slice(bytes.as_slice()).expect("bundle should decode");
    assert_eq!(decoded, bundle);
}

#[test]
fn llm_bundle_contract_version_is_explicit() {
    assert_eq!(LLM_BUNDLE_VERSION, "llm_bundle/1.0");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core llm_bundle_round_trip_json -- --nocapture`  
Expected: FAIL with missing `llm_bundle` module/types.

**Step 3: Write minimal implementation**

```rust
pub const LLM_BUNDLE_VERSION: &str = "llm_bundle/1.0";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmBundle {
    pub version: String,
    pub request: BundleRequest,
    pub figma: BundleFigmaContext,
    pub artifacts: BundleArtifacts,
    pub instructions: BundleInstructions,
    pub tool_contract: BundleToolContract,
}
```

Add supporting structs for request, artifact refs, instruction payloads, and tool contract.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core llm_bundle_round_trip_json llm_bundle_contract_version_is_explicit`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/llm_bundle.rs crates/core/src/lib.rs
git commit -m "feat(core): add llm bundle contract types"
```

### Task 2: Implement Deterministic Bundle Builder in Core

**Files:**
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/src/tests.rs`
- Modify: `crates/core/src/llm_bundle.rs`
- Create: `crates/core/src/hash.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn prepare_llm_bundle_in_workspace_writes_bundle_artifact() {
    let workspace_root = temp_workspace_root();
    seed_full_fixture_pipeline(workspace_root.as_path());

    let result = prepare_llm_bundle_in_workspace(
        workspace_root.as_path(),
        PrepareLlmBundleRequest {
            figma_url: "https://www.figma.com/design/abc/Screen?node-id=1-2".to_string(),
            target: "react-tailwind".to_string(),
            intent: "Generate production-ready login screen".to_string(),
        },
    )
    .expect("bundle should build");

    assert_eq!(result, "output/agent/llm_bundle.json");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core prepare_llm_bundle_in_workspace_writes_bundle_artifact -- --nocapture`  
Expected: FAIL with missing API.

**Step 3: Write minimal implementation**

Implement:

1. `PrepareLlmBundleRequest` + `prepare_llm_bundle_in_workspace(...)`.
2. Required artifact checks for:
   - `output/specs/ui_spec.ron`
   - `output/agent/agent_context.json`
   - `output/agent/search_index.json`
   - `output/assets/asset_manifest.json`
3. Deterministic artifact hash helper in `hash.rs`.
4. Instruction ingestion from:
   - `.codex/SKILLS.md`
   - `docs/agent-playbook.md`
   - `docs/figma-ui-coder.md`
5. Bundle write to `output/agent/llm_bundle.json`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core prepare_llm_bundle_in_workspace_writes_bundle_artifact`  
Expected: PASS and bundle path stable.

**Step 5: Commit**

```bash
git add crates/core/src/lib.rs crates/core/src/tests.rs crates/core/src/llm_bundle.rs crates/core/src/hash.rs
git commit -m "feat(core): add deterministic llm bundle builder"
```

### Task 3: Add CLI `prepare-llm-bundle` Command Surface

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`
- Test: `crates/cli/tests/commands.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn prepare_llm_bundle_subcommand_writes_bundle_path() {
    let output = run_cli([
        "prepare-llm-bundle",
        "--figma-url",
        "https://www.figma.com/design/abc/Screen?node-id=1-2",
        "--target",
        "react-tailwind",
        "--intent",
        "Generate login screen code",
    ]);
    assert!(output.stdout.contains("artifact=output/agent/llm_bundle.json"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-cli prepare_llm_bundle_subcommand_writes_bundle_path -- --nocapture`  
Expected: FAIL with unknown subcommand.

**Step 3: Write minimal implementation**

Add new subcommand:

```rust
PrepareLlmBundle {
    #[arg(long)] figma_url: String,
    #[arg(long)] target: String,
    #[arg(long)] intent: String,
    #[arg(long, value_enum, default_value_t = OutputMode::Text)] output: OutputMode,
}
```

Wire it to `core::prepare_llm_bundle(...)` and emit text/json output consistent with existing commands.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-cli prepare_llm_bundle_subcommand_writes_bundle_path`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/src/main.rs crates/cli/tests/commands.rs crates/cli/tests/integration_smoke.rs
git commit -m "feat(cli): add prepare-llm-bundle command"
```

### Task 4: Add Agent Runner Abstraction with Deterministic Mock

**Files:**
- Create: `crates/core/src/agent_runner.rs`
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/src/tests.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn generate_ui_with_mock_runner_writes_generated_output() {
    let workspace_root = temp_workspace_root();
    seed_bundle_artifact(workspace_root.as_path());

    let result = generate_ui_in_workspace(
        workspace_root.as_path(),
        GenerateUiRequest {
            bundle_path: "output/agent/llm_bundle.json".to_string(),
        },
        &MockAgentRunner::default(),
    )
    .expect("generate ui should succeed");

    assert!(result.generated_paths.iter().any(|p| p.starts_with("output/generated/")));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core generate_ui_with_mock_runner_writes_generated_output -- --nocapture`  
Expected: FAIL with missing runner and generate APIs.

**Step 3: Write minimal implementation**

Implement:

1. `AgentRunner` trait.
2. `MockAgentRunner` for deterministic test output.
3. `GenerateUiRequest`, `GenerateUiResult`, and runner payload/response types.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core generate_ui_with_mock_runner_writes_generated_output`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/agent_runner.rs crates/core/src/lib.rs crates/core/src/tests.rs
git commit -m "feat(core): add agent runner abstraction and mock backend"
```

### Task 5: Implement `generate-ui` Core Workflow with Report Guarantees

**Files:**
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/src/tests.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn generate_ui_in_workspace_always_emits_warning_and_trace_artifacts() {
    let workspace_root = temp_workspace_root();
    seed_bundle_artifact(workspace_root.as_path());

    generate_ui_in_workspace(
        workspace_root.as_path(),
        GenerateUiRequest { bundle_path: "output/agent/llm_bundle.json".to_string() },
        &MockAgentRunner::default(),
    )
    .expect("generation should succeed");

    assert!(workspace_root.join("output/reports/generation_warnings.json").is_file());
    assert!(workspace_root.join("output/reports/generation_trace.json").is_file());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core generate_ui_in_workspace_always_emits_warning_and_trace_artifacts -- --nocapture`  
Expected: FAIL because generation workflow does not write report artifacts yet.

**Step 3: Write minimal implementation**

Implement:

1. Bundle load + validation.
2. Target output directory creation.
3. Generated file writes based on runner response.
4. Guaranteed warning/trace artifact write/update path (even if empty payloads).
5. Actionable errors for missing bundle or invalid contract.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core generate_ui_in_workspace_always_emits_warning_and_trace_artifacts`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/lib.rs crates/core/src/tests.rs
git commit -m "feat(core): add generate-ui workflow with report outputs"
```

### Task 6: Add CLI `generate-ui` Command Surface

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`
- Test: `crates/cli/tests/commands.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn generate_ui_subcommand_reports_generated_artifact_paths() {
    let output = run_cli(["generate-ui", "--bundle", "output/agent/llm_bundle.json"]);
    assert!(output.stdout.contains("output/generated/"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-cli generate_ui_subcommand_reports_generated_artifact_paths -- --nocapture`  
Expected: FAIL with unknown subcommand.

**Step 3: Write minimal implementation**

Add:

```rust
GenerateUi {
    #[arg(long)] bundle: String,
    #[arg(long, value_enum, default_value_t = OutputMode::Text)] output: OutputMode,
}
```

Wire command to `core::generate_ui(...)` using default runner configuration and structured output formatting.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-cli generate_ui_subcommand_reports_generated_artifact_paths`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/src/main.rs crates/cli/tests/commands.rs crates/cli/tests/integration_smoke.rs
git commit -m "feat(cli): add generate-ui command"
```

### Task 7: Add End-to-End Fixture Integration Coverage for New Workflow

**Files:**
- Create: `crates/cli/tests/e2e_agent_workflow.rs`
- Modify: `scripts/verify_workspace.sh`
- Test: `crates/cli/tests/e2e_agent_workflow.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn prepare_then_generate_agent_workflow_fixture_path() {
    run_cli(["prepare-llm-bundle", "--figma-url", "https://www.figma.com/design/abc/Screen?node-id=1-2", "--target", "react-tailwind", "--intent", "Generate login UI"]);
    run_cli(["generate-ui", "--bundle", "output/agent/llm_bundle.json"]);
    assert!(std::path::Path::new("output/generated/react-tailwind").exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-cli --test e2e_agent_workflow -- --nocapture`  
Expected: FAIL before new commands/workflow are fully wired.

**Step 3: Write minimal implementation**

1. Add fixture-safe E2E test helpers.
2. Assert required outputs exist:
   - `output/agent/llm_bundle.json`
   - `output/generated/<target>/...`
   - warnings + trace reports
3. Extend `scripts/verify_workspace.sh` to run this new test target.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-cli --test e2e_agent_workflow`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/tests/e2e_agent_workflow.rs scripts/verify_workspace.sh
git commit -m "test(cli): add e2e coverage for prepare and generate workflow"
```

### Task 8: Update Operator Documentation

**Files:**
- Modify: `README.md`
- Modify: `crates/cli/README.md`
- Modify: `docs/agent-playbook.md`
- Modify: `docs/figma-ui-coder.md`

**Step 1: Write the failing check**

```bash
rg -n "prepare-llm-bundle|generate-ui|llm_bundle.json" README.md crates/cli/README.md docs/agent-playbook.md docs/figma-ui-coder.md
```

Expected: FAIL (missing new flow references before update).

**Step 2: Run check to verify it fails**

Run: `rg -n "prepare-llm-bundle|generate-ui|llm_bundle.json" README.md crates/cli/README.md docs/agent-playbook.md docs/figma-ui-coder.md`  
Expected: no matches for at least one required term.

**Step 3: Write minimal implementation**

Document:

1. New two-command operator flow.
2. Bundle contract purpose and location.
3. Skills + playbook instruction ingestion policy.
4. Report artifact expectations.

**Step 4: Run check to verify it passes**

Run: `rg -n "prepare-llm-bundle|generate-ui|llm_bundle.json" README.md crates/cli/README.md docs/agent-playbook.md docs/figma-ui-coder.md`  
Expected: PASS with matches in all files.

**Step 5: Commit**

```bash
git add README.md crates/cli/README.md docs/agent-playbook.md docs/figma-ui-coder.md
git commit -m "docs(workspace): add prepare-and-generate agent workflow docs"
```

### Task 9: Final Verification and Phase Close-Out

**Files:**
- Modify: `docs/plans/boards/2026-03-05-phase-15-end2end-agent-workflow-board.md` (new board if phase tracking is used)

**Step 1: Run format + build + tests**

Run:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
bash scripts/verify_workspace.sh
```

Expected: PASS.

**Step 2: Record verification evidence**

Capture command outputs in board notes and final handoff summary.

**Step 3: Commit phase close-out metadata**

```bash
git add docs/plans/boards/2026-03-05-phase-15-end2end-agent-workflow-board.md
git commit -m "docs(docs): close phase 15 end-to-end agent workflow board"
```

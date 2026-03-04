# UI Blueprint and LLM Generation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a target-agnostic `ui_blueprint.yaml` artifact and shift UI code generation to LLM-driven tooling while keeping deterministic Rust stages authoritative through `build-spec`.

**Architecture:** Keep `fetch -> normalize -> infer-layout -> build-spec` deterministic, then add a deterministic projection stage `build-ui-blueprint` that emits `output/specs/ui_blueprint.yaml` alongside `ui_spec.json`. Add separate LLM tooling paths: a deterministic bundle-prep command and a direct model generation command that consumes the Blueprint artifact and writes target UI files with run metadata for audit.

**Tech Stack:** Rust 2024 workspace, `serde`, `serde_json`, `serde_yaml`, `clap`, `reqwest` (blocking client), `sha2`, `thiserror`.

---

Use `@test-driven-development` for each task, `@systematic-debugging` if any command output is unexpected, and `@verification-before-completion` before claiming milestone completion.

### Task 1: Create `ui_blueprint` Crate and Core Contract

**Files:**
- Create: `crates/ui_blueprint/Cargo.toml`
- Create: `crates/ui_blueprint/src/lib.rs`

**Step 1: Write the failing test**

```bash
cargo test -p ui_blueprint
```

Expected: FAIL with unknown package `ui_blueprint`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui_blueprint`
Expected: non-zero exit (`package ID specification 'ui_blueprint' did not match any packages`).

**Step 3: Write minimal implementation**

```toml
# crates/ui_blueprint/Cargo.toml
[package]
name = "ui_blueprint"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "2.0"
```

```rust
// crates/ui_blueprint/src/lib.rs
#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiBlueprint {
    pub version: String,
    pub document: BlueprintDocument,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<BlueprintComponent>,
    pub screens: Vec<BlueprintScreen>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<BlueprintAsset>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<BlueprintWarning>,
}
```

Add unit tests in `lib.rs`:
1. serialize/deserialize round-trip.
2. version defaults to `ui_blueprint/1.0`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui_blueprint`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui_blueprint
git commit -m "feat(core): add ui blueprint contract crate"
```

### Task 2: Implement `ui_spec -> ui_blueprint` Mapping

**Files:**
- Modify: `crates/ui_blueprint/Cargo.toml`
- Modify: `crates/ui_blueprint/src/lib.rs`

**Step 1: Write the failing test**

Add test in `crates/ui_blueprint/src/lib.rs`:

```rust
#[test]
fn build_blueprint_from_ui_spec_maps_root_layout_and_warnings() {
    let spec = ui_spec::UiSpec::default();
    let blueprint = build_ui_blueprint(&spec);
    assert_eq!(blueprint.version, "ui_blueprint/1.0");
    assert!(!blueprint.screens.is_empty());
}
```

Expected: compile FAIL because `build_ui_blueprint` does not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui_blueprint build_blueprint_from_ui_spec_maps_root_layout_and_warnings -- --nocapture`
Expected: compile error for missing function or dependency.

**Step 3: Write minimal implementation**

Add dependency:

```toml
# crates/ui_blueprint/Cargo.toml
ui_spec = { path = "../ui_spec" }
```

Add mapper:

```rust
pub fn build_ui_blueprint(spec: &ui_spec::UiSpec) -> UiBlueprint {
    UiBlueprint {
        version: "ui_blueprint/1.0".to_string(),
        document: BlueprintDocument::from_source(&spec.source),
        components: Vec::new(),
        screens: vec![BlueprintScreen {
            id: format!("screen/{}", spec.source.root_node_id),
            name: spec.root.name.clone(),
            root: map_node(&spec.root),
        }],
        assets: Vec::new(),
        warnings: spec.warnings.iter().map(BlueprintWarning::from).collect(),
    }
}
```

Use inferred semantic layout names (`stack_v`, `stack_h`, `overlay`, `absolute`, `scroll`).

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui_blueprint`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui_blueprint
git commit -m "feat(core): map ui spec to ui blueprint"
```

### Task 3: Add Stable YAML Emission for UI Blueprint

**Files:**
- Modify: `crates/ui_blueprint/src/lib.rs`

**Step 1: Write the failing test**

Add test:

```rust
#[test]
fn to_yaml_is_stable_for_identical_blueprint() {
    let blueprint = sample_blueprint();
    let first = blueprint.to_yaml_string().unwrap();
    let second = blueprint.to_yaml_string().unwrap();
    assert_eq!(first, second);
    assert!(first.contains("version: ui_blueprint/1.0"));
}
```

Expected: FAIL because `to_yaml_string` is missing.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui_blueprint to_yaml_is_stable_for_identical_blueprint -- --nocapture`
Expected: compile error.

**Step 3: Write minimal implementation**

```rust
impl UiBlueprint {
    pub fn to_yaml_string(&self) -> Result<String, BlueprintError> {
        serde_yaml::to_string(self).map_err(BlueprintError::SerializeYaml)
    }
}
```

Define:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    #[error("yaml serialization error: {0}")]
    SerializeYaml(serde_yaml::Error),
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui_blueprint`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui_blueprint
git commit -m "feat(core): add deterministic ui blueprint yaml emission"
```

### Task 4: Add `build-ui-blueprint` Stage in Orchestrator

**Files:**
- Modify: `crates/orchestrator/Cargo.toml`
- Modify: `crates/orchestrator/src/lib.rs`

**Step 1: Write the failing test**

Update/add test in `crates/orchestrator/src/lib.rs`:

```rust
#[test]
fn run_stage_build_ui_blueprint_writes_yaml_artifact() {
    let workspace_root = unique_test_workspace_root("run_stage_build_ui_blueprint");
    run_stage_in_workspace("fetch", workspace_root.as_path()).unwrap();
    run_stage_in_workspace("normalize", workspace_root.as_path()).unwrap();
    run_stage_in_workspace("infer-layout", workspace_root.as_path()).unwrap();
    run_stage_in_workspace("build-spec", workspace_root.as_path()).unwrap();
    let result = run_stage_in_workspace("build-ui-blueprint", workspace_root.as_path()).unwrap();
    assert_eq!(result.artifact_path, Some("output/specs/ui_blueprint.yaml".to_string()));
}
```

Expected: FAIL because stage is unknown.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator run_stage_build_ui_blueprint_writes_yaml_artifact -- --nocapture`
Expected: FAIL with unknown stage.

**Step 3: Write minimal implementation**

1. Add dependency:

```toml
# crates/orchestrator/Cargo.toml
ui_blueprint = { path = "../ui_blueprint" }
```

2. Add constants:

```rust
const BLUEPRINT_ARTIFACT_RELATIVE_PATH: &str = "output/specs/ui_blueprint.yaml";
```

3. Add stage definition and handler:

```rust
PipelineStageDefinition { name: "build-ui-blueprint", output_dir: "output/specs" }
```

```rust
"build-ui-blueprint" => Some(run_build_ui_blueprint_stage(workspace_root)?),
```

4. Implement `run_build_ui_blueprint_stage`:
   - read `ui_spec.json`
   - map with `ui_blueprint::build_ui_blueprint`
   - write YAML artifact to `output/specs/ui_blueprint.yaml`

5. Keep `gen-swiftui` callable as legacy/deprecated path, but remove it from default `run_all` stage list.

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator`
Expected: PASS, including updated stage order assertions.

**Step 5: Commit**

```bash
git add crates/orchestrator/Cargo.toml crates/orchestrator/src/lib.rs
git commit -m "feat(core): add build-ui-blueprint pipeline stage"
```

### Task 5: Update CLI Surface for Blueprint-First Flow

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Step 1: Write the failing test**

Update `generate_success_smoke` expected output to include `build-ui-blueprint` and exclude `gen-swiftui`.

Expected output segment:

```text
... stage=build-spec output=output/specs artifact=output/specs/ui_spec.json
stage=build-ui-blueprint output=output/specs artifact=output/specs/ui_blueprint.yaml
stage=export-assets ...
```

Expected: FAIL until CLI/orchestrator output is updated.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli --test integration_smoke generate_success_smoke -- --nocapture`
Expected: assertion failure on stdout mismatch.

**Step 3: Write minimal implementation**

1. Add command variant:

```rust
BuildUiBlueprint,
```

2. Map stage names:

```rust
Command::BuildUiBlueprint => "build-ui-blueprint",
```

3. Keep `GenSwiftui` behind hidden/deprecated command metadata (not part of default docs path).

4. Ensure `generate` output lines reflect orchestrator stage order with Blueprint artifact.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cli --test integration_smoke`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/src/main.rs crates/cli/tests/integration_smoke.rs
git commit -m "feat(cli): add ui blueprint stage commands"
```

### Task 6: Add Deterministic LLM Bundle Builder Crate

**Files:**
- Create: `crates/llm_bundle/Cargo.toml`
- Create: `crates/llm_bundle/src/lib.rs`
- Create: `output/llm/.gitkeep`

**Step 1: Write the failing test**

```bash
cargo test -p llm_bundle
```

Expected: FAIL with unknown package.

**Step 2: Run test to verify it fails**

Run: `cargo test -p llm_bundle`
Expected: non-zero exit for missing package.

**Step 3: Write minimal implementation**

Add crate with:
1. `LlmBundle` struct containing:
   - target
   - blueprint path/hash
   - asset manifest path/hash
   - warnings summary
   - prompt template version
2. `build_bundle(...)` function.
3. deterministic hash helper using `sha2`.
4. JSON serialization helper for artifact writing.

Add tests:
1. same inputs produce same hash.
2. warning summary is preserved.

**Step 4: Run test to verify it passes**

Run: `cargo test -p llm_bundle`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/llm_bundle output/llm/.gitkeep
git commit -m "feat(core): add llm bundle builder crate"
```

### Task 7: Add `prepare-llm-bundle` Orchestrator/CLI Command

**Files:**
- Modify: `crates/orchestrator/Cargo.toml`
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Step 1: Write the failing test**

Add CLI smoke test:

```rust
#[test]
fn prepare_llm_bundle_success_smoke() {
    let workspace_root = unique_cli_workspace_root("prepare_llm_bundle_success_smoke");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(workspace_root.as_path())
        .args(["run-stage", "prepare-llm-bundle"])
        .output()
        .unwrap();
    assert!(out.status.success());
}
```

Expected: FAIL because stage is unknown.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli --test integration_smoke prepare_llm_bundle_success_smoke -- --nocapture`
Expected: exit code `2` / unknown stage.

**Step 3: Write minimal implementation**

1. Add `llm_bundle` dependency in orchestrator.
2. Add stage `prepare-llm-bundle` (outside default `generate` order).
3. Implement stage handler:
   - require `output/specs/ui_blueprint.yaml`
   - require `output/assets/asset_manifest.json`
   - write `output/llm/llm_bundle.json`
4. Add CLI top-level command:

```rust
PrepareLlmBundle,
```

5. Ensure `run-stage prepare-llm-bundle` reports deterministic artifact path.

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator && cargo test -p cli --test integration_smoke`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/orchestrator/Cargo.toml crates/orchestrator/src/lib.rs crates/cli/src/main.rs crates/cli/tests/integration_smoke.rs
git commit -m "feat(cli): add prepare-llm-bundle command"
```

### Task 8: Add Direct LLM UI Generation Crate

**Files:**
- Create: `crates/llm_codegen/Cargo.toml`
- Create: `crates/llm_codegen/src/lib.rs`

**Step 1: Write the failing test**

```bash
cargo test -p llm_codegen
```

Expected: FAIL with unknown package.

**Step 2: Run test to verify it fails**

Run: `cargo test -p llm_codegen`
Expected: non-zero exit for missing package.

**Step 3: Write minimal implementation**

Implement:
1. `GenerateUiRequest` with:
   - `model`
   - `target`
   - `bundle_path`
   - `output_dir`
   - `api_key`
   - optional `api_base_url` for tests
2. `generate_ui(...)` that:
   - reads bundle JSON
   - builds strict prompt requiring JSON file list output
   - calls model endpoint with `reqwest::blocking`
   - parses response into `GeneratedFiles`
   - writes generated files under requested output dir
3. run record writer:
   - `output/llm/run-<timestamp>.json`

Add tests:
1. request validation (missing key/model/target).
2. mock-server success parses files and writes output.

**Step 4: Run test to verify it passes**

Run: `cargo test -p llm_codegen`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/llm_codegen
git commit -m "feat(core): add direct llm ui generation crate"
```

### Task 9: Add `generate-ui` CLI Command

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`
- Modify: `crates/orchestrator/Cargo.toml`
- Modify: `crates/orchestrator/src/lib.rs`

**Step 1: Write the failing test**

Add CLI test for usage validation:

```rust
#[test]
fn generate_ui_requires_api_key() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(["generate-ui", "--target", "swiftui", "--model", "gpt-5"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("OPENAI_API_KEY"));
}
```

Expected: FAIL because command does not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli --test integration_smoke generate_ui_requires_api_key -- --nocapture`
Expected: non-zero / unknown subcommand.

**Step 3: Write minimal implementation**

1. Add CLI command:

```rust
GenerateUi {
    #[arg(long)] target: String,
    #[arg(long, default_value = "gpt-5")] model: String,
    #[arg(long)] api_key: Option<String>,
    #[arg(long)] bundle_path: Option<String>,
    #[arg(long)] output_dir: Option<String>,
    #[arg(long, hide = true)] api_base_url: Option<String>,
}
```

2. Resolve API key from `--api-key` or `OPENAI_API_KEY`.
3. Call orchestrator entrypoint that wraps `llm_codegen::generate_ui`.
4. Emit machine-readable JSON output with run metadata path.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cli --test integration_smoke`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/src/main.rs crates/cli/tests/integration_smoke.rs crates/orchestrator/Cargo.toml crates/orchestrator/src/lib.rs
git commit -m "feat(cli): add generate-ui llm command"
```

### Task 10: Documentation and Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/proposal.md`
- Modify: `docs/plans/2026-03-04-figma-swiftui-generator.md`

**Step 1: Write the failing test**

Run docs consistency checks by asserting new commands/artifacts are discoverable:

```bash
rg -n "ui_blueprint.yaml|build-ui-blueprint|prepare-llm-bundle|generate-ui" README.md docs/proposal.md docs/plans/2026-03-04-figma-swiftui-generator.md
```

Expected: partial/missing matches before docs updates.

**Step 2: Run test to verify it fails**

Run the `rg` command above.
Expected: missing sections in one or more docs.

**Step 3: Write minimal implementation**

Update docs to reflect:
1. dual artifacts (`ui_spec.json` + `ui_blueprint.yaml`).
2. deterministic default stage order without `gen-swiftui`.
3. LLM tooling commands and required env vars.
4. run metadata/audit expectations.

**Step 4: Run verification**

Run:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/verify_workspace.sh
```

Expected: PASS.

**Step 5: Commit**

```bash
git add README.md docs/proposal.md docs/plans/2026-03-04-figma-swiftui-generator.md
git commit -m "docs: document ui blueprint and llm generation workflow"
```


# Figma Node Tree to SwiftUI Generator Implementation Plan

> **Update (2026-03-04):** The implementation has pivoted to a Blueprint-first + LLM flow. See `docs/plans/2026-03-04-ui-blueprint-llm-generation.md` for current execution details, including `build-ui-blueprint`, `prepare-llm-bundle`, `generate-ui`, and `output/specs/ui_blueprint.yaml`.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bootstrap a deterministic Rust 2024 workspace that can execute the Figma-node-to-SwiftUI pipeline in explicit stages, with stable contracts and verification-first development.

**Architecture:** Use a crate-per-stage workspace where `cli` orchestrates commands through an `orchestrator` boundary and each domain crate owns one responsibility (fetch, normalize, infer, spec, AST, codegen, assets, report). Build schema-first contracts and deterministic stage wiring before feature depth, then expand stage internals with strict warning/report surfacing.

**Tech Stack:** Rust 2024, Cargo workspace, `serde`, `serde_json`, `schemars` (later tasks), `thiserror`, `miette`, `tracing`, `tokio`, `reqwest`, `insta`, `proptest`.

---

Use `@test-driven-development` for each implementation task, `@systematic-debugging` if any command output is unexpected, and `@verification-before-completion` before claiming the milestone is complete.

### Task 1: Create Workspace Root

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Test: `Cargo.toml` (validate with Cargo)

**Step 1: Write the failing test**

```bash
cargo metadata --no-deps
```

Expected: FAIL because no workspace manifest exists yet.

**Step 2: Run test to verify it fails**

Run: `cargo metadata --no-deps`
Expected: non-zero exit with missing `Cargo.toml`.

**Step 3: Write minimal implementation**

```toml
# Cargo.toml
[workspace]
members = ["crates/*"]
resolver = "2"
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
```

**Step 4: Run test to verify it passes**

Run: `cargo metadata --no-deps`
Expected: PASS with JSON metadata output.

**Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml
git commit -m "chore: initialize rust workspace root"
```

### Task 2: Add Core Crate Layout

**Files:**
- Create: `crates/cli/Cargo.toml`
- Create: `crates/cli/src/main.rs`
- Create: `crates/figma_client/Cargo.toml`
- Create: `crates/figma_client/src/lib.rs`
- Create: `crates/figma_normalizer/Cargo.toml`
- Create: `crates/figma_normalizer/src/lib.rs`
- Create: `crates/layout_infer/Cargo.toml`
- Create: `crates/layout_infer/src/lib.rs`
- Create: `crates/ui_spec/Cargo.toml`
- Create: `crates/ui_spec/src/lib.rs`
- Create: `crates/swiftui_ast/Cargo.toml`
- Create: `crates/swiftui_ast/src/lib.rs`
- Create: `crates/swiftui_codegen/Cargo.toml`
- Create: `crates/swiftui_codegen/src/lib.rs`
- Create: `crates/asset_pipeline/Cargo.toml`
- Create: `crates/asset_pipeline/src/lib.rs`
- Create: `crates/review_report/Cargo.toml`
- Create: `crates/review_report/src/lib.rs`
- Create: `crates/orchestrator/Cargo.toml`
- Create: `crates/orchestrator/src/lib.rs`
- Test: all crate manifests and roots

**Step 1: Write the failing test**

```bash
cargo check --workspace
```

Expected: FAIL because workspace member crates do not exist.

**Step 2: Run test to verify it fails**

Run: `cargo check --workspace`
Expected: non-zero exit with missing member path errors.

**Step 3: Write minimal implementation**

```toml
# crates/figma_client/Cargo.toml (pattern repeated for library crates)
[package]
name = "figma_client"
version = "0.1.0"
edition = "2024"

[dependencies]
```

```rust
// crates/figma_client/src/lib.rs (pattern repeated for library crates)
#![forbid(unsafe_code)]
```

```toml
# crates/cli/Cargo.toml
[package]
name = "cli"
version = "0.1.0"
edition = "2024"

[dependencies]
orchestrator = { path = "../orchestrator" }
```

```rust
// crates/cli/src/main.rs
fn main() {
    println!("figma-swiftui cli");
}
```

**Step 4: Run test to verify it passes**

Run: `cargo check --workspace`
Expected: PASS for all workspace members.

**Step 5: Commit**

```bash
git add Cargo.toml crates/
git commit -m "chore: scaffold rust pipeline crates"
```

### Task 3: Add Output Artifact Directories

**Files:**
- Create: `output/raw/.gitkeep`
- Create: `output/normalized/.gitkeep`
- Create: `output/inferred/.gitkeep`
- Create: `output/specs/.gitkeep`
- Create: `output/swift/.gitkeep`
- Create: `output/assets/.gitkeep`
- Create: `output/reports/.gitkeep`
- Test: directory existence check

**Step 1: Write the failing test**

```bash
test -d output/raw && test -d output/reports
```

Expected: FAIL before directories exist.

**Step 2: Run test to verify it fails**

Run: `test -d output/raw && test -d output/reports`
Expected: non-zero exit status.

**Step 3: Write minimal implementation**

Create all output directories and minimal keep files so they persist in git.

**Step 4: Run test to verify it passes**

Run: `test -d output/raw && test -d output/reports`
Expected: PASS (exit status 0).

**Step 5: Commit**

```bash
git add output/
git commit -m "chore: add pipeline output directories"
```

### Task 4: Define Initial Contract Types

**Files:**
- Modify: `crates/ui_spec/src/lib.rs`
- Modify: `crates/swiftui_ast/src/lib.rs`
- Modify: `crates/review_report/src/lib.rs`
- Test: per-crate unit tests in each crate root

**Step 1: Write the failing test**

```rust
#[test]
fn ui_spec_round_trip() {
    let spec = UiSpec::default();
    let json = serde_json::to_string(&spec).unwrap();
    let back: UiSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec, back);
}
```

Expected: FAIL because contract structs and serde derives do not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui_spec ui_spec_round_trip -- --nocapture`
Expected: compile failure for missing symbols/traits.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UiSpec {
    pub spec_version: String,
}

impl Default for UiSpec {
    fn default() -> Self {
        Self {
            spec_version: "1.0".to_string(),
        }
    }
}
```

Apply equivalent minimal versioned roots for `SwiftUiAst` and `ReviewReport`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui_spec -p swiftui_ast -p review_report`
Expected: PASS for serialization round-trip tests.

**Step 5: Commit**

```bash
git add crates/ui_spec crates/swiftui_ast crates/review_report
git commit -m "feat(core): add versioned contract roots"
```

### Task 5: Wire Orchestrator Stage Interfaces

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs` unit tests

**Step 1: Write the failing test**

```rust
#[test]
fn stages_are_reported_in_order() {
    let stages = pipeline_stage_names();
    assert_eq!(
        stages,
        vec![
            "fetch",
            "normalize",
            "infer-layout",
            "build-spec",
            "gen-swiftui",
            "export-assets",
            "report"
        ]
    );
}
```

Expected: FAIL because no stage API exists yet.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator stages_are_reported_in_order -- --nocapture`
Expected: compile error for missing function.

**Step 3: Write minimal implementation**

```rust
pub fn pipeline_stage_names() -> Vec<&'static str> {
    vec![
        "fetch",
        "normalize",
        "infer-layout",
        "build-spec",
        "gen-swiftui",
        "export-assets",
        "report",
    ]
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/orchestrator
git commit -m "feat(core): add pipeline stage contract"
```

### Task 6: Add CLI Command Surface Skeleton

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/Cargo.toml`
- Test: `crates/cli/tests/commands.rs`

**Step 1: Write the failing test**

```rust
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
```

Expected: FAIL because clap surface is not implemented.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli help_lists_pipeline_subcommands -- --nocapture`
Expected: FAIL assertion.

**Step 3: Write minimal implementation**

Use `clap` derive with subcommands:

```rust
#[derive(clap::Subcommand)]
enum Command {
    Fetch,
    Normalize,
    InferLayout,
    BuildSpec,
    GenSwiftui,
    ExportAssets,
    Report,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli
git commit -m "feat(cli): add pipeline subcommand skeleton"
```

### Task 7: Establish Shared Error Baseline

**Files:**
- Modify: `crates/orchestrator/Cargo.toml`
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs` tests

**Step 1: Write the failing test**

```rust
#[test]
fn unsupported_feature_is_classified() {
    let err = PipelineError::UnsupportedFeature("mask".into());
    assert!(err.to_string().contains("unsupported"));
}
```

Expected: FAIL because `PipelineError` does not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator unsupported_feature_is_classified -- --nocapture`
Expected: compile failure.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/orchestrator
git commit -m "feat(core): add pipeline error baseline"
```

### Task 8: Deterministic Serialization Guard

**Files:**
- Modify: `crates/ui_spec/src/lib.rs`
- Test: `crates/ui_spec/src/lib.rs` tests

**Step 1: Write the failing test**

```rust
#[test]
fn serialization_is_stable() {
    let spec = UiSpec::default();
    let a = serde_json::to_vec_pretty(&spec).unwrap();
    let b = serde_json::to_vec_pretty(&spec).unwrap();
    assert_eq!(a, b);
}
```

Expected: FAIL until canonical representation and derives are complete.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui_spec serialization_is_stable -- --nocapture`
Expected: FAIL.

**Step 3: Write minimal implementation**

Ensure deterministic root fields and add explicit constructor defaults for fixed values.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui_spec`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui_spec
git commit -m "test(core): add deterministic serialization guard"
```

### Task 9: Add Workspace Smoke Verification Script

**Files:**
- Create: `scripts/verify_workspace.sh`
- Test: command invocation from repository root

**Step 1: Write the failing test**

```bash
./scripts/verify_workspace.sh
```

Expected: FAIL because script does not exist.

**Step 2: Run test to verify it fails**

Run: `./scripts/verify_workspace.sh`
Expected: shell "No such file or directory".

**Step 3: Write minimal implementation**

```bash
#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

**Step 4: Run test to verify it passes**

Run: `bash scripts/verify_workspace.sh`
Expected: PASS when workspace checks/tests pass.

**Step 5: Commit**

```bash
git add scripts/verify_workspace.sh
git commit -m "chore: add workspace verification script"
```

### Task 10: Add First Integration Smoke Test

**Files:**
- Create: `crates/cli/tests/integration_smoke.rs`
- Test: integration test command

**Step 1: Write the failing test**

```rust
#[test]
fn cli_help_smoke() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(out.status.success());
}
```

Expected: FAIL before final CLI wiring stabilizes.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli --test integration_smoke -- --nocapture`
Expected: FAIL.

**Step 3: Write minimal implementation**

Finalize `cli` binary argument parser and exit behavior to ensure help returns success.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cli --test integration_smoke`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/cli/tests/integration_smoke.rs crates/cli/src/main.rs
git commit -m "test(cli): add help integration smoke test"
```

### Task 11: Prepare Next Stage Contract TODO Map

**Files:**
- Create: `docs/plans/next-stage-contract-map.md`
- Test: markdown lint/readability check (manual)

**Step 1: Write the failing test**

Manual expectation: no explicit mapping document exists for stage-specific contract expansion.

**Step 2: Run test to verify it fails**

Run: `test -f docs/plans/next-stage-contract-map.md`
Expected: non-zero exit status.

**Step 3: Write minimal implementation**

Document exact structs to add next:

1. Normalized node graph types.
2. Layout decision record with confidence and alternatives.
3. Asset manifest schema.
4. Review report warning categories.

**Step 4: Run test to verify it passes**

Run: `test -f docs/plans/next-stage-contract-map.md`
Expected: PASS.

**Step 5: Commit**

```bash
git add docs/plans/next-stage-contract-map.md
git commit -m "docs: add next stage contract map"
```

### Task 12: Final Milestone Verification

**Files:**
- Modify: none (verification task)
- Test: whole workspace

**Step 1: Write the failing test**

```bash
bash scripts/verify_workspace.sh
```

Expected: if any previous task regressed, this fails.

**Step 2: Run test to verify it fails**

If it fails, stop and use `@systematic-debugging`.

**Step 3: Write minimal implementation**

Fix only failing task outputs. Avoid new scope.

**Step 4: Run test to verify it passes**

Run:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
```

Expected: all PASS.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: complete bootstrap milestone verification"
```

## Definition of Done

1. Workspace contains all proposal-aligned crates and artifact directories.
2. `cargo check --workspace` passes with Rust 2024 crates.
3. Versioned contract roots compile and serialize deterministically.
4. CLI exposes the stage command surface.
5. Verification script and smoke tests run cleanly.

# Project Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a usable end-to-end Rust pipeline that takes Figma node JSON input and produces deterministic SwiftUI files, asset manifests, and review reports through the CLI.

**Architecture:** Build stage behavior incrementally behind the existing crate boundaries (`figma_client` -> `figma_normalizer` -> `layout_infer` -> `ui_spec` -> `swiftui_ast` -> `swiftui_codegen` -> `asset_pipeline` -> `review_report`), and wire execution in `orchestrator` and `cli` with explicit artifact I/O and deterministic serialization.

**Tech Stack:** Rust 2024 workspace, `serde`/`serde_json`, `clap`, `thiserror`, fixture-based integration tests, workspace verification script.

---

## Current Status Snapshot (2026-03-04)

1. Workspace scaffolding is complete and clean on `main`.
2. Stage contracts and CLI command surface exist (`stages`, `run-stage`), but stage behavior is still placeholder-level.
3. Contract roots exist in core crates, but end-to-end data flow is not implemented.
4. Verification currently passes:
   - `cargo check --workspace`
   - `cargo test --workspace`
   - `bash scripts/verify_workspace.sh`

## Ready-to-Use Definition

The project is considered ready to use when all items below are true:

1. A user can run one CLI command against fixture input and generate:
   - normalized JSON
   - layout decision JSON
   - UI spec JSON
   - SwiftUI source files
   - asset manifest JSON
   - review report JSON
2. All pipeline stages are executed through `orchestrator` (not ad hoc in CLI).
3. Outputs are deterministic across repeated runs with identical input.
4. Unsupported or low-confidence cases are surfaced in review artifacts (never silently dropped).
5. Workspace verification and fixture e2e tests pass in CI/local.
6. README quickstart is sufficient for a new contributor to run the pipeline.

## Execution Rules

1. Use phase boards in `docs/plans/boards/` and keep task states `[ ]`, `[~]`, `[x]`.
2. Keep one logical task per commit and follow `docs/commit-style.md`.
3. Merge each completed phase into `main`, verify on merged `main`, then branch next phase from latest `main`.
4. Run verification before claiming any task complete.

## Phase 06: Implement Input Stages (Fetch + Normalize)

**Board:** `docs/plans/boards/2026-03-04-phase-06-input-stages-board.md`

### Task P6-T1: Implement Figma fetch client primitives

**Files:**
- Modify: `crates/figma_client/src/lib.rs`
- Modify: `crates/figma_client/Cargo.toml`
- Test: `crates/figma_client/src/lib.rs`

**Deliverable:** typed request/response models and fetch entrypoint for raw node-tree payloads (fixture-driven first).

**Verification:** `cargo test -p figma_client`

**Commit:** `feat(figma-client): add fetch request and response contracts`

### Task P6-T2: Add fetch stage artifact writing

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Create: `output/raw/.gitkeep` (already exists, keep)
- Test: `crates/orchestrator/src/lib.rs`

**Deliverable:** `run_stage("fetch")` writes deterministic raw snapshot JSON to `output/raw/` and returns output metadata.

**Verification:** `cargo test -p orchestrator`

**Commit:** `feat(orchestrator): persist fetch stage snapshot artifacts`

### Task P6-T3: Implement raw-to-normalized translation

**Files:**
- Modify: `crates/figma_normalizer/src/lib.rs`
- Modify: `crates/figma_normalizer/Cargo.toml`
- Test: `crates/figma_normalizer/src/lib.rs`

**Deliverable:** normalizer function that maps raw Figma fixture JSON into `NormalizedDocument` and emits explicit warnings for unsupported node fields.

**Verification:** `cargo test -p figma_normalizer`

**Commit:** `feat(normalizer): add raw figma to normalized document transform`

### Task P6-T4: Wire normalize stage in orchestrator

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs`, `crates/cli/tests/integration_smoke.rs`

**Deliverable:** `run-stage normalize` reads latest raw artifact and writes normalized artifact into `output/normalized/`.

**Verification:** `cargo test -p orchestrator && cargo test -p cli --test integration_smoke`

**Commit:** `feat(orchestrator): execute normalize stage with artifact handoff`

### Task P6-T5: Phase verification and merge

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-06-input-stages-board.md`

**Verification:** `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh`

**Commit:** `chore(workspace): complete phase 06 input stage verification`

## Phase 07: Implement Inference + UI Spec Build

**Board:** `docs/plans/boards/2026-03-04-phase-07-infer-spec-board.md`

### Task P7-T1: Implement deterministic layout heuristics

**Files:**
- Modify: `crates/layout_infer/src/lib.rs`
- Test: `crates/layout_infer/src/lib.rs`

**Deliverable:** rules-first inference API selecting `v_stack`, `h_stack`, `overlay`, or `absolute` with confidence and alternatives.

**Verification:** `cargo test -p layout_infer`

**Commit:** `feat(layout): add rules-first layout inference engine`

### Task P7-T2: Expand UI spec model to usable tree

**Files:**
- Modify: `crates/ui_spec/src/lib.rs`
- Test: `crates/ui_spec/src/lib.rs`

**Deliverable:** versioned, serializable UI spec tree with root node, layout, style, and children contracts.

**Verification:** `cargo test -p ui_spec`

**Commit:** `feat(ui-spec): expand ui spec tree contracts`

### Task P7-T3: Build spec from normalized + inferred inputs

**Files:**
- Modify: `crates/ui_spec/src/lib.rs`
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/ui_spec/src/lib.rs`, `crates/orchestrator/src/lib.rs`

**Deliverable:** build-spec stage writes deterministic `output/specs/*.json` from upstream artifacts.

**Verification:** `cargo test -p ui_spec && cargo test -p orchestrator`

**Commit:** `feat(orchestrator): add build-spec stage artifact generation`

### Task P7-T4: Promote inference warnings to report-ready shape

**Files:**
- Modify: `crates/review_report/src/lib.rs`
- Modify: `crates/layout_infer/src/lib.rs`
- Test: `crates/review_report/src/lib.rs`

**Deliverable:** conversion helpers mapping layout warnings into `ReviewWarning` categories/severities.

**Verification:** `cargo test -p review_report && cargo test -p layout_infer`

**Commit:** `feat(review-report): map layout warnings into review categories`

### Task P7-T5: Phase verification and merge

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-07-infer-spec-board.md`

**Verification:** `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh`

**Commit:** `chore(workspace): complete phase 07 inference and spec verification`

## Phase 08: Implement SwiftUI AST + Codegen + Assets

**Board:** `docs/plans/boards/2026-03-04-phase-08-codegen-assets-board.md`

### Task P8-T1: Expand SwiftUI AST model

**Files:**
- Modify: `crates/swiftui_ast/src/lib.rs`
- Test: `crates/swiftui_ast/src/lib.rs`

**Deliverable:** typed SwiftUI AST nodes for container, text, image, spacer, and modifier primitives.

**Verification:** `cargo test -p swiftui_ast`

**Commit:** `feat(swiftui-ast): add typed swiftui view node contracts`

### Task P8-T2: Implement deterministic SwiftUI renderer

**Files:**
- Modify: `crates/swiftui_codegen/src/lib.rs`
- Modify: `crates/swiftui_codegen/Cargo.toml`
- Test: `crates/swiftui_codegen/src/lib.rs`

**Deliverable:** renderer API that converts AST into deterministic Swift source strings with stable indentation/newlines.

**Verification:** `cargo test -p swiftui_codegen`

**Commit:** `feat(swiftui-codegen): add deterministic swift source renderer`

### Task P8-T3: Implement spec-to-ast mapping and swift output stage

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `crates/swiftui_ast/src/lib.rs`
- Modify: `crates/swiftui_codegen/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs`, `crates/cli/tests/integration_smoke.rs`

**Deliverable:** `run-stage gen-swiftui` reads UI spec and writes `.swift` output into `output/swift/`.

**Verification:** `cargo test -p orchestrator && cargo test -p cli --test integration_smoke`

**Commit:** `feat(orchestrator): add spec to swiftui generation stage`

### Task P8-T4: Implement asset export manifest generation

**Files:**
- Modify: `crates/asset_pipeline/src/lib.rs`
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/asset_pipeline/src/lib.rs`, `crates/orchestrator/src/lib.rs`

**Deliverable:** `run-stage export-assets` writes deterministic `AssetManifest` to `output/assets/` using normalized/spec references.

**Verification:** `cargo test -p asset_pipeline && cargo test -p orchestrator`

**Commit:** `feat(assets): add deterministic export manifest builder`

### Task P8-T5: Phase verification and merge

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-08-codegen-assets-board.md`

**Verification:** `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh`

**Commit:** `chore(workspace): complete phase 08 codegen and assets verification`

## Phase 09: End-to-End Pipeline Execution + Reporting

**Board:** `docs/plans/boards/2026-03-04-phase-09-e2e-pipeline-board.md`

### Task P9-T1: Implement review report stage aggregation

**Files:**
- Modify: `crates/review_report/src/lib.rs`
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/review_report/src/lib.rs`, `crates/orchestrator/src/lib.rs`

**Deliverable:** `run-stage report` aggregates warnings from normalization/inference/assets into a single `ReviewReport` artifact in `output/reports/`.

**Verification:** `cargo test -p review_report && cargo test -p orchestrator`

**Commit:** `feat(review-report): add cross-stage report aggregation`

### Task P9-T2: Add orchestrator run-all pipeline entrypoint

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs`

**Deliverable:** sequential `run_all` API executing all stages with explicit stage results and error boundaries.

**Verification:** `cargo test -p orchestrator`

**Commit:** `feat(orchestrator): add run-all pipeline execution`

### Task P9-T3: Add CLI generate command

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Deliverable:** CLI command to run full pipeline (`generate`) with text/json output and deterministic exit codes.

**Verification:** `cargo test -p cli`

**Commit:** `feat(cli): add generate command for full pipeline`

### Task P9-T4: Add fixture-based end-to-end test suite

**Files:**
- Create: `crates/cli/tests/fixtures/` (representative raw input)
- Create: `crates/cli/tests/e2e_generate.rs`
- Test: `crates/cli/tests/e2e_generate.rs`

**Deliverable:** e2e test proving one command generates all required artifacts and expected report/warning behavior.

**Verification:** `cargo test -p cli --test e2e_generate`

**Commit:** `test(cli): add fixture e2e generate coverage`

### Task P9-T5: Determinism regression tests for generated artifacts

**Files:**
- Modify: `crates/cli/tests/e2e_generate.rs`

**Deliverable:** repeated-run assertions proving byte-stable output files for identical fixture input.

**Verification:** `cargo test -p cli --test e2e_generate`

**Commit:** `test(workspace): assert deterministic e2e artifact generation`

### Task P9-T6: Phase verification and merge

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-09-e2e-pipeline-board.md`

**Verification:** `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh`

**Commit:** `chore(workspace): complete phase 09 e2e pipeline verification`

## Phase 10: Usability Hardening + Documentation

**Board:** `docs/plans/boards/2026-03-04-phase-10-usability-release-board.md`

### Task P10-T1: Publish root usage docs

**Files:**
- Create: `README.md`
- Modify: `docs/proposal.md` (link to usage if needed)

**Deliverable:** quickstart (inputs, commands, outputs, troubleshooting) and scope/non-scope clarity.

**Verification:** docs self-check walkthrough from clean terminal.

**Commit:** `docs(workspace): add quickstart and usage guide`

### Task P10-T2: Add sample workflow command documentation

**Files:**
- Modify: `docs/plans/boards/README.md`
- Modify: `README.md`

**Deliverable:** explicit command matrix for `stages`, `run-stage`, and `generate` in text/json modes.

**Verification:** docs self-check + command copy/paste validation.

**Commit:** `docs(cli): document stage and generate command flows`

### Task P10-T3: Harden CLI/user-facing errors

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/cli/tests/commands.rs`

**Deliverable:** actionable errors for missing artifacts, invalid input, and unsupported features; stable non-zero exit codes.

**Verification:** `cargo test -p cli && cargo test -p orchestrator`

**Commit:** `fix(cli): improve actionable pipeline error messages`

### Task P10-T4: Expand verification script to include e2e smoke

**Files:**
- Modify: `scripts/verify_workspace.sh`

**Deliverable:** verification script includes critical e2e command/test in addition to unit/integration checks.

**Verification:** `bash scripts/verify_workspace.sh`

**Commit:** `chore(ci): include e2e smoke in workspace verification`

### Task P10-T5: Final readiness gate and merge

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-10-usability-release-board.md`

**Verification:**
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `bash scripts/verify_workspace.sh`
- Manual quickstart run from `README.md`

**Commit:** `chore(workspace): declare mvp pipeline ready for use`

## Final Acceptance Checklist

1. Full pipeline command works on fixture input with deterministic outputs.
2. Every stage writes artifacts into the expected `output/*` directory.
3. Warnings are preserved in report artifacts for unsupported/ambiguous features.
4. All phase boards are fully `[x]` and merged to `main` in order.
5. Workspace verification and e2e tests pass on merged `main`.
6. README enables first-run success without tribal knowledge.

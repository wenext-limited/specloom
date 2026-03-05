# Phase 15 Parallel Board: End-to-End Agent Workflow

**Phase ID:** `P15`
**Goal:** Ship a reproducible `prepare-llm-bundle` + `generate-ui` workflow that converts Figma URL + intent into generated target code with warning/trace artifacts.
**Source Plan:** `docs/plans/2026-03-05-end2end-agent-workflow.md`
**Last Updated:** `2026-03-05 18:38 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P15-T1 | Add `llm_bundle` contract types in core | codex | - | `crates/core/src/llm_bundle.rs`, `crates/core/src/lib.rs` | `cargo test -p specloom-core llm_bundle_ -- --nocapture` | `4ca6ec9` | Started 2026-03-05 18:27 CST; completed 2026-03-05 18:29 CST |
| [x] | P15-T2 | Implement deterministic bundle builder and artifact hashing | codex | P15-T1 | `crates/core/src/lib.rs`, `crates/core/src/tests.rs`, `crates/core/src/hash.rs`, `crates/core/src/llm_bundle.rs` | `cargo test -p specloom-core prepare_llm_bundle_in_workspace_writes_bundle_artifact -- --nocapture` | `9422260` | Started 2026-03-05 18:29 CST; completed 2026-03-05 18:33 CST |
| [x] | P15-T3 | Add CLI `prepare-llm-bundle` command surface | codex | P15-T2 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p specloom-cli prepare_llm_bundle_subcommand_writes_bundle_path -- --nocapture` | `4d90db4` | Started 2026-03-05 18:33 CST; completed 2026-03-05 18:35 CST |
| [x] | P15-T4 | Add agent runner abstraction with deterministic mock backend | codex | P15-T2 | `crates/core/src/agent_runner.rs`, `crates/core/src/lib.rs`, `crates/core/src/tests.rs` | `cargo test -p specloom-core generate_ui_with_mock_runner_writes_generated_output -- --nocapture` | `c26b369` | Started 2026-03-05 18:35 CST; completed 2026-03-05 18:37 CST |
| [x] | P15-T5 | Implement core `generate-ui` workflow with warning/trace guarantees | codex | P15-T4 | `crates/core/src/lib.rs`, `crates/core/src/tests.rs` | `cargo test -p specloom-core generate_ui_in_workspace_always_emits_warning_and_trace_artifacts -- --nocapture` | `c7e18a5` | Started 2026-03-05 18:37 CST; completed 2026-03-05 18:38 CST |
| [~] | P15-T6 | Add CLI `generate-ui` command surface | codex | P15-T5 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p specloom-cli generate_ui_subcommand_reports_generated_artifact_paths` | - | Started 2026-03-05 18:38 CST |
| [ ] | P15-T7 | Add fixture E2E test for prepare+generate workflow and verify script gate | unassigned | P15-T3,P15-T6 | `crates/cli/tests/e2e_agent_workflow.rs`, `scripts/verify_workspace.sh` | `cargo test -p specloom-cli --test e2e_agent_workflow` | - | |
| [ ] | P15-T8 | Update operator docs for new workflow | unassigned | P15-T3,P15-T6 | `README.md`, `crates/cli/README.md`, `docs/agent-playbook.md`, `docs/figma-ui-coder.md` | `rg -n "prepare-llm-bundle|generate-ui|llm_bundle.json" README.md crates/cli/README.md docs/agent-playbook.md docs/figma-ui-coder.md` | - | |
| [ ] | P15-T9 | Phase verification and close-out on branch | unassigned | P15-T7,P15-T8 | `docs/plans/boards/2026-03-05-phase-15-end2end-agent-workflow-board.md` | `cargo fmt --all --check && cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | |

## Parallelization Rules

1. `P15-T2`, `P15-T4`, and `P15-T5` all touch `crates/core/src/lib.rs`; keep them sequential.
2. `P15-T3` and `P15-T6` both touch CLI files and stay sequential.
3. `P15-T7` and `P15-T8` can run in parallel after `P15-T3` and `P15-T6` are complete.
4. Do not claim tasks with unmet dependencies.
5. Do not mark `[x]` without commit hash and passing verification evidence.

## Phase Exit Criteria

1. Every task row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is ready for merge to `main`, followed by merged-main verification.

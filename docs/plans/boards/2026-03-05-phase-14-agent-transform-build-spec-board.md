# Phase 14 Parallel Board: Agent Transform Build-Spec

**Phase ID:** `P14`
**Goal:** Remove `infer-layout` from the active path and make `build-spec` agent-driven via `transform_plan.json` with explicit `child_policy`.
**Source Plan:** `docs/plans/2026-03-05-agent-transform-build-spec.md`
**Last Updated:** `2026-03-05 04:16 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P14-T1 | Publish plan+board for `transform_plan` + `child_policy` contract | codex | - | `docs/plans/2026-03-05-agent-transform-build-spec.md`, `docs/plans/boards/2026-03-05-phase-14-agent-transform-build-spec-board.md` | `rg -n "transform_plan|child_policy|replace_with" docs/plans/2026-03-05-agent-transform-build-spec.md docs/plans/boards/2026-03-05-phase-14-agent-transform-build-spec-board.md` | `cf512fd` | Started 2026-03-05 04:09 CST; completed 2026-03-05 04:11 CST; verification command passed |
| [x] | P14-T2 | Remove `infer-layout` from default orchestration and fixture expectations | codex | P14-T1 | `crates/orchestrator/src/lib.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs`, `crates/cli/tests/fixtures/generate_expected_output.json` | `cargo test -p orchestrator && cargo test -p cli` | `b12e854` | Started 2026-03-05 04:13 CST; completed 2026-03-05 04:14 CST; verified `cargo test -p orchestrator && cargo test -p cli`; manual `run-stage infer-layout` removed |
| [x] | P14-T3 | Emit `pre_layout.ron` and `node_map.json` during `build-spec` | codex | P14-T2 | `crates/orchestrator/src/lib.rs`, `crates/ui_spec/src/build.rs` | `cargo test -p orchestrator && cargo test -p ui_spec` | `7a04066` | Started 2026-03-05 04:16 CST; completed 2026-03-05 04:16 CST; verified `cargo test -p orchestrator && cargo test -p ui_spec`; `node_map.json` keys are `BTreeMap`-ordered |
| [x] | P14-T4 | Add transform plan contract with `child_policy` modes and validation | codex | P14-T3 | `crates/ui_spec/src/lib.rs`, `crates/ui_spec/src/transform_plan.rs`, `crates/ui_spec/src/tests.rs` | `cargo test -p ui_spec` | `b9bca31` | Started 2026-03-05 04:18 CST; completed 2026-03-05 04:42 CST; verified `cargo test -p ui_spec`; modes: `keep`, `drop`, `replace_with` |
| [ ] | P14-T5 | Apply agent transform plan mechanically to produce final `ui_spec.ron` | unassigned | P14-T4 | `crates/ui_spec/src/build.rs`, `crates/ui_spec/src/tests.rs` | `cargo test -p ui_spec` | - | No post-AI semantic re-inference |
| [ ] | P14-T6 | Wire agent transform-plan production in `build-spec` pipeline | unassigned | P14-T5 | `crates/orchestrator/src/lib.rs`, `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs` | `cargo test -p orchestrator && cargo test -p cli` | - | Persist `output/specs/transform_plan.json` |
| [ ] | P14-T7 | Regenerate `agent_context.json` from final transformed spec and update docs | unassigned | P14-T6 | `crates/orchestrator/src/lib.rs`, `README.md`, `docs/agent-playbook.md` | `cargo test -p orchestrator && cargo test -p cli` | - | Document end-to-end artifact order |
| [ ] | P14-T8 | Phase verification and merged-main close-out | unassigned | P14-T7 | `docs/plans/boards/2026-03-05-phase-14-agent-transform-build-spec-board.md` | `cargo check --workspace && cargo test --workspace` | - | Merge to `main` and record evidence |

## Parallelization Rules

1. `P14-T3` and `P14-T4` are sequential because `P14-T4` depends on new artifacts from `P14-T3`.
2. `P14-T5` and `P14-T6` are sequential because both touch transform application surfaces.
3. `P14-T7` can run after `P14-T6`; avoid concurrent edits to `crates/orchestrator/src/lib.rs`.
4. Do not claim tasks with unmet dependencies.
5. Do not mark `[x]` without commit hash and verification evidence.

## Phase Exit Criteria

1. Every task is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. Work is merged into `main`.
5. Verification is re-run on merged `main` and recorded in `P14-T8`.

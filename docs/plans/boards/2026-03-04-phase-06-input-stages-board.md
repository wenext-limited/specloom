# Phase 06 Parallel Board: Input Stages

**Phase ID:** `P6`
**Goal:** Implement fetch and normalize stages with deterministic artifact handoff.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 15:10 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P6-T1 | Implement Figma fetch client primitives | codex | - | `crates/figma_client/src/lib.rs`, `crates/figma_client/Cargo.toml` | `cargo test -p figma_client` | `4c0f6cb` | Started 2026-03-04 15:01 CST; fixture-driven request/response contracts implemented and verified |
| [x] | P6-T2 | Add fetch stage artifact writing | codex | P6-T1 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | `bb65e01` | Started 2026-03-04 15:08 CST; writes deterministic `output/raw/fetch_snapshot.json` and returns artifact metadata |
| [ ] | P6-T3 | Implement raw-to-normalized translation | unassigned | P6-T1 | `crates/figma_normalizer/src/lib.rs`, `crates/figma_normalizer/Cargo.toml` | `cargo test -p figma_normalizer` | - | Emit warnings for unsupported fields |
| [ ] | P6-T4 | Wire normalize stage in orchestrator + CLI smoke | unassigned | P6-T2,P6-T3 | `crates/orchestrator/src/lib.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p orchestrator && cargo test -p cli --test integration_smoke` | - | Read `output/raw/`, write `output/normalized/` |
| [ ] | P6-T5 | Phase verification and closeout | unassigned | P6-T4 | `docs/plans/boards/2026-03-04-phase-06-input-stages-board.md` | `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge phase to `main` after pass |

## Parallelization Rules

1. `P6-T2` and `P6-T3` can run in parallel only after `P6-T1` is complete.
2. Keep one owner per `[~]` task.
3. Do not mark `[x]` without task-scoped verification output.
4. Update row status and commit hash in the same commit when possible.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is merged into `main`.

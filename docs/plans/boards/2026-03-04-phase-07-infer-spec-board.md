# Phase 07 Parallel Board: Inference and Spec

**Phase ID:** `P7`
**Goal:** Implement deterministic layout inference and build-spec artifact generation.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 15:10 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P7-T1 | Implement deterministic layout heuristics | codex | - | `crates/layout_infer/src/lib.rs` | `cargo test -p layout_infer` | `8e9955f` | Started 2026-03-04 15:25 CST; added rules-first metadata/geometry inference with deterministic decisions and warnings |
| [x] | P7-T2 | Expand UI spec tree contracts | codex | - | `crates/ui_spec/src/lib.rs` | `cargo test -p ui_spec` | `30ba1e7` | Expanded spec root/source/node/warning contracts with deterministic child ordering |
| [x] | P7-T3 | Build spec from normalized + inferred inputs | codex | P7-T1,P7-T2 | `crates/ui_spec/src/lib.rs`, `crates/orchestrator/src/lib.rs` | `cargo test -p ui_spec && cargo test -p orchestrator` | `b1db0a8` | Started 2026-03-04 15:35 CST; infer + build-spec stages now write deterministic inferred/spec artifacts |
| [x] | P7-T4 | Map inference warnings into review warning types | codex | P7-T1 | `crates/review_report/src/lib.rs`, `crates/layout_infer/src/lib.rs` | `cargo test -p review_report && cargo test -p layout_infer` | `42f3884` | Started 2026-03-04 15:39 CST; added deterministic mapping from inference warnings to review categories/severities |
| [ ] | P7-T5 | Phase verification and closeout | unassigned | P7-T3,P7-T4 | `docs/plans/boards/2026-03-04-phase-07-infer-spec-board.md` | `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge phase to `main` after pass |

## Parallelization Rules

1. `P7-T1` and `P7-T2` can run in parallel.
2. `P7-T3` waits for both `P7-T1` and `P7-T2`.
3. `P7-T4` can run after `P7-T1`.
4. One owner per `[~]` task and no overlapping file ownership.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is merged into `main`.

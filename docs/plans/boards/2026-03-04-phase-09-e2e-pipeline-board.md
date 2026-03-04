# Phase 09 Parallel Board: End-to-End Pipeline

**Phase ID:** `P9`
**Goal:** Add full pipeline execution, reporting, and fixture e2e coverage.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 16:08 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P9-T1 | Implement review report stage aggregation | codex | - | `crates/review_report/src/lib.rs`, `crates/orchestrator/src/lib.rs` | `cargo test -p review_report && cargo test -p orchestrator` | `cd1ed7b` | Started 2026-03-04 15:44 CST; report stage now aggregates normalization/inference/asset warnings into deterministic review artifact |
| [x] | P9-T2 | Add orchestrator run-all API | codex | P9-T1 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | `0998183` | Started 2026-03-04 15:49 CST; added sequential run-all execution with ordered results and stage-error boundaries |
| [x] | P9-T3 | Add CLI generate command | codex | P9-T2 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | `ff679ab` | Started 2026-03-04 15:53 CST; generate now runs full pipeline with deterministic text/json output and stable error exit handling |
| [x] | P9-T4 | Add fixture e2e generate test | codex | P9-T3 | `crates/cli/tests/fixtures/`, `crates/cli/tests/e2e_generate.rs` | `cargo test -p cli --test e2e_generate` | `2da7e19` | Started 2026-03-04 16:00 CST; fixture-driven e2e generate test now verifies all artifact families and report warning summary |
| [x] | P9-T5 | Add deterministic rerun assertions | codex | P9-T4 | `crates/cli/tests/e2e_generate.rs` | `cargo test -p cli --test e2e_generate` | `030a6da` | Started 2026-03-04 16:06 CST; added repeated-run byte-equality assertions for all generated artifact outputs |
| [ ] | P9-T6 | Phase verification and closeout | unassigned | P9-T5 | `docs/plans/boards/2026-03-04-phase-09-e2e-pipeline-board.md` | `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge phase to `main` after pass |

## Parallelization Rules

1. `P9-T1` must complete before `P9-T2`.
2. `P9-T2` must complete before `P9-T3`.
3. `P9-T4` and `P9-T5` are sequential to avoid fixture contract drift.
4. Use strict fixture ownership during active `[~]` tasks.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is merged into `main`.

# Phase 09 Parallel Board: End-to-End Pipeline

**Phase ID:** `P9`
**Goal:** Add full pipeline execution, reporting, and fixture e2e coverage.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 15:48 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P9-T1 | Implement review report stage aggregation | codex | - | `crates/review_report/src/lib.rs`, `crates/orchestrator/src/lib.rs` | `cargo test -p review_report && cargo test -p orchestrator` | `cd1ed7b` | Started 2026-03-04 15:44 CST; report stage now aggregates normalization/inference/asset warnings into deterministic review artifact |
| [ ] | P9-T2 | Add orchestrator run-all API | unassigned | P9-T1 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Return ordered stage results |
| [ ] | P9-T3 | Add CLI generate command | unassigned | P9-T2 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | - | Deterministic text/json outputs and exit codes |
| [ ] | P9-T4 | Add fixture e2e generate test | unassigned | P9-T3 | `crates/cli/tests/fixtures/`, `crates/cli/tests/e2e_generate.rs` | `cargo test -p cli --test e2e_generate` | - | Assert all output artifact families are created |
| [ ] | P9-T5 | Add deterministic rerun assertions | unassigned | P9-T4 | `crates/cli/tests/e2e_generate.rs` | `cargo test -p cli --test e2e_generate` | - | Byte-equal outputs across repeated runs |
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

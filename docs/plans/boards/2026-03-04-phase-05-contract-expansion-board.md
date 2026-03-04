# Phase 05 Parallel Board: Contract Expansion

**Phase ID:** `P5`
**Goal:** Expand deterministic, versioned stage contracts across core crates from the next-stage map.
**Source Plan:** `docs/plans/next-stage-contract-map.md`
**Last Updated:** `2026-03-04 14:39 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P5-T1 | Add/verify normalized node graph contract roots and round-trip tests | codex-worker-p5t1 | - | `crates/figma_normalizer/src/lib.rs` | `cargo test -p figma_normalizer` | `238365a` | Added explicit version constants and expanded deterministic ordering tests |
| [x] | P5-T2 | Add/verify layout decision record contract and warning structures | codex-worker-p5t2 | - | `crates/layout_infer/src/lib.rs` | `cargo test -p layout_infer` | `b119606` | Enforced contract shape with unknown-field rejection and explicit warning/ordering tests |
| [x] | P5-T3 | Add/verify asset manifest contract and ordered entry behavior | codex-worker-p5t3 | - | `crates/asset_pipeline/src/lib.rs` | `cargo test -p asset_pipeline` | `8ec0dcf` | Added generation metadata contract, hashed filename field, and deterministic shape tests |
| [x] | P5-T4 | Add/verify review warning categories and summary counters | codex-worker-p5t4 | - | `crates/review_report/src/lib.rs` | `cargo test -p review_report` | `d307194` | Pre-seeded summary counters for all category/severity variants with stable enum value tests |
| [x] | P5-T5 | Add cross-crate contract consistency checks and workspace verification | codex | P5-T1,P5-T2,P5-T3,P5-T4 | `crates/*/src/lib.rs`, `scripts/verify_workspace.sh` | `cargo check --workspace && cargo test --workspace` | `c0fd3ae` | Added explicit cross-crate contract checks in `scripts/verify_workspace.sh`; workspace verification passed |
| [x] | P5-T6 | Update docs with contract expansion evidence and close phase board | codex | P5-T5 | `docs/plans/next-stage-contract-map.md`, `docs/plans/boards/2026-03-04-phase-05-contract-expansion-board.md` | docs self-check | `d2e6d30` | Added phase evidence summary, follow-up scope, and finalized board status |

## Parallelization Rules

1. `P5-T1` through `P5-T4` are parallelizable if file ownership boundaries are preserved.
2. Keep one owner per `[~]` row.
3. Do not claim `P5-T5` until all dependent rows are `[x]`.
4. Add commit hash and verification evidence before marking `[x]`.

## Phase Exit Criteria

1. Every task is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. Work is merged into `main`.

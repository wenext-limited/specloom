# Phase 05 Parallel Board: Contract Expansion

**Phase ID:** `P5`
**Goal:** Expand deterministic, versioned stage contracts across core crates from the next-stage map.
**Source Plan:** `docs/plans/next-stage-contract-map.md`
**Last Updated:** `2026-03-04 14:27 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [ ] | P5-T1 | Add/verify normalized node graph contract roots and round-trip tests | unassigned | - | `crates/figma_normalizer/src/lib.rs` | `cargo test -p figma_normalizer` | - | Keep schema/version fields explicit and stable |
| [ ] | P5-T2 | Add/verify layout decision record contract and warning structures | unassigned | - | `crates/layout_infer/src/lib.rs` | `cargo test -p layout_infer` | - | Include confidence + alternatives + warnings |
| [ ] | P5-T3 | Add/verify asset manifest contract and ordered entry behavior | unassigned | - | `crates/asset_pipeline/src/lib.rs` | `cargo test -p asset_pipeline` | - | Preserve deterministic entry order |
| [ ] | P5-T4 | Add/verify review warning categories and summary counters | unassigned | - | `crates/review_report/src/lib.rs` | `cargo test -p review_report` | - | Keep category/severity output deterministic |
| [ ] | P5-T5 | Add cross-crate contract consistency checks and workspace verification | unassigned | P5-T1,P5-T2,P5-T3,P5-T4 | `crates/*/src/lib.rs`, `scripts/verify_workspace.sh` | `cargo check --workspace && cargo test --workspace` | - | Integrate and verify whole-workspace contract stability |
| [ ] | P5-T6 | Update docs with contract expansion evidence and close phase board | unassigned | P5-T5 | `docs/plans/next-stage-contract-map.md`, `docs/plans/boards/2026-03-04-phase-05-contract-expansion-board.md` | docs self-check | - | Record commits, verification, and follow-up scope |

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

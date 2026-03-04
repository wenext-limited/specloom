# Phase 08 Parallel Board: Codegen and Assets

**Phase ID:** `P8`
**Goal:** Implement SwiftUI AST/codegen and asset manifest stages.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 15:10 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P8-T1 | Expand SwiftUI AST model | codex | - | `crates/swiftui_ast/src/lib.rs` | `cargo test -p swiftui_ast` | `c87f124` | Started 2026-03-04 15:52 CST; added typed core nodes/modifiers with deterministic serialization tests |
| [x] | P8-T2 | Implement deterministic SwiftUI renderer | codex | P8-T1 | `crates/swiftui_codegen/src/lib.rs`, `crates/swiftui_codegen/Cargo.toml` | `cargo test -p swiftui_codegen` | `c004328` | Started 2026-03-04 15:54 CST; deterministic renderer with stable formatting/modifier emission implemented |
| [x] | P8-T3 | Wire gen-swiftui stage from spec to files | codex | P8-T1,P8-T2 | `crates/orchestrator/src/lib.rs`, `crates/swiftui_ast/src/lib.rs`, `crates/swiftui_codegen/src/lib.rs` | `cargo test -p orchestrator && cargo test -p cli --test integration_smoke` | `cd304db` | Started 2026-03-04 15:58 CST; gen-swiftui now maps spec->AST->Swift source and writes deterministic `.swift` artifact |
| [ ] | P8-T4 | Implement export-assets stage manifest builder | unassigned | - | `crates/asset_pipeline/src/lib.rs`, `crates/orchestrator/src/lib.rs` | `cargo test -p asset_pipeline && cargo test -p orchestrator` | - | Deterministic `output/assets/*.json` |
| [ ] | P8-T5 | Phase verification and closeout | unassigned | P8-T3,P8-T4 | `docs/plans/boards/2026-03-04-phase-08-codegen-assets-board.md` | `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge phase to `main` after pass |

## Parallelization Rules

1. `P8-T1` and `P8-T4` can run in parallel.
2. `P8-T2` depends on `P8-T1`.
3. `P8-T3` depends on `P8-T1` and `P8-T2`.
4. Keep file ownership boundaries explicit before dispatch.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is merged into `main`.

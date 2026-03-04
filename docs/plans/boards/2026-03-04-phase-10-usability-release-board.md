# Phase 10 Parallel Board: Usability and Release

**Phase ID:** `P10`
**Goal:** Make pipeline usable by contributors through docs, hardened errors, and final verification gates.
**Source Plan:** `docs/plans/2026-03-04-project-readiness-implementation.md`
**Last Updated:** `2026-03-04 16:18 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P10-T1 | Add root quickstart README | codex | - | `README.md`, `docs/proposal.md` | docs self-check walkthrough | `71780cb` | Started 2026-03-04 15:57 CST; added root quickstart with input/output scope and validated documented CLI commands |
| [x] | P10-T2 | Document full CLI workflow | codex | P10-T1 | `README.md`, `docs/plans/boards/README.md` | docs self-check + command copy/paste check | `5841ec7` | Started 2026-03-04 16:04 CST; documented workflow matrix and validated all documented command variants via copy/paste |
| [x] | P10-T3 | Harden CLI and orchestrator error UX | codex | - | `crates/cli/src/main.rs`, `crates/orchestrator/src/lib.rs`, `crates/cli/tests/commands.rs` | `cargo test -p cli && cargo test -p orchestrator` | `<pending>` | Started 2026-03-04 16:12 CST; added actionable error guidance for unknown stage, missing artifacts, and workspace IO failures |
| [ ] | P10-T4 | Extend verification script with e2e smoke | unassigned | P10-T3 | `scripts/verify_workspace.sh` | `bash scripts/verify_workspace.sh` | - | Keep script deterministic and CI-friendly |
| [ ] | P10-T5 | Final readiness gate and closeout | unassigned | P10-T2,P10-T4 | `docs/plans/boards/2026-03-04-phase-10-usability-release-board.md` | `cargo fmt --all --check && cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Manual README quickstart run required |

## Parallelization Rules

1. `P10-T1` and `P10-T3` can run in parallel.
2. `P10-T2` depends on `P10-T1`.
3. `P10-T4` depends on `P10-T3`.
4. `P10-T5` waits for `P10-T2` and `P10-T4`.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo fmt --all --check` passes.
3. `cargo check --workspace` passes.
4. `cargo test --workspace` passes.
5. `bash scripts/verify_workspace.sh` passes.
6. Phase branch is merged into `main`.

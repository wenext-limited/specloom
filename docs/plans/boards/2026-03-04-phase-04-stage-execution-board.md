# Phase 04 Parallel Board: Stage Execution

**Phase ID:** `P4`
**Goal:** Add a minimal stage execution contract and CLI surface that can run or inspect a selected pipeline stage deterministically.
**Source Plan:** `docs/plans/2026-03-04-figma-swiftui-generator.md`
**Last Updated:** `2026-03-04 14:10 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P4-T0 | Create phase board and parallel workflow docs | codex | - | `docs/plans/boards/README.md`, `docs/plans/templates/parallel-phase-board-template.md`, `AGENTS.md` | docs self-check | `<pending>` | Board workflow bootstrapped |
| [~] | P4-T1 | Add orchestrator stage execution result contract | codex | P4-T0 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Current active task |
| [ ] | P4-T2 | Add CLI `run-stage` command wired to orchestrator contract | unassigned | P4-T1 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs` | `cargo test -p cli` | - | Can run in parallel with docs once P4-T1 lands |
| [ ] | P4-T3 | Add CLI output mode flag (`text`/`json`) for `stages` and `run-stage` | unassigned | P4-T2 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs` | `cargo test -p cli` | - | Keep output deterministic |
| [ ] | P4-T4 | Add integration coverage for stage command contract | unassigned | P4-T2 | `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli --test integration_smoke` | - | Validate command contract end-to-end |
| [ ] | P4-T5 | Update user docs for stage commands and board workflow | unassigned | P4-T4 | `docs/plans/boards/README.md` or `docs/` | docs self-check | - | Finalize phase docs |

## Phase Exit Criteria

1. All rows are `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. Phase changes are pushed on `main`.

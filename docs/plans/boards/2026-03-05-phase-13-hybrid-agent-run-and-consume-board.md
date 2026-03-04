# Phase 13 Parallel Board: Hybrid Agent Run-and-Consume

**Phase ID:** `P13`
**Goal:** Deliver strict-hybrid agent tooling with deterministic fuzzy node lookup and a default stateless run-and-consume CLI execution model.
**Source Plan:** `docs/plans/2026-03-05-figma-hybrid-agent-generation.md`
**Last Updated:** `2026-03-05 03:28 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P13-T1 | Codify run-and-consume runtime model in approved design + implementation plan | codex | - | `docs/plans/2026-03-05-figma-hybrid-agent-generation-design.md`, `docs/plans/2026-03-05-figma-hybrid-agent-generation.md`, `docs/plans/boards/2026-03-05-phase-13-hybrid-agent-run-and-consume-board.md` | `rg -n "run-and-consume|daemon|background process" docs/plans/2026-03-05-figma-hybrid-agent-generation-design.md docs/plans/2026-03-05-figma-hybrid-agent-generation.md docs/plans/boards/2026-03-05-phase-13-hybrid-agent-run-and-consume-board.md` | `3afd0b0` | Started 2026-03-05 03:14 CST; completed 2026-03-05 03:16 CST; verification command passed |
| [x] | P13-T2 | Create `agent_context` crate and workspace registration | codex | P13-T1 | `Cargo.toml`, `crates/agent_context/Cargo.toml`, `crates/agent_context/src/lib.rs` | `cargo test -p agent_context` | `27668aa` | Started 2026-03-05 03:18 CST; completed 2026-03-05 03:22 CST; verified `cargo test -p agent_context` |
| [x] | P13-T3 | Add JSON contracts for agent context, search index, warnings, and trace | codex | P13-T2 | `crates/agent_context/src/lib.rs` | `cargo test -p agent_context` | `fb15fa1` | Started 2026-03-05 03:23 CST; completed 2026-03-05 03:23 CST; verified `cargo test -p agent_context` |
| [x] | P13-T4 | Implement deterministic fuzzy ranking + thresholds in `agent_context` | codex | P13-T3 | `crates/agent_context/src/lib.rs`, `crates/agent_context/src/search.rs` | `cargo test -p agent_context` | `f8d2dd8` | Started 2026-03-05 03:24 CST; completed 2026-03-05 03:26 CST; verified `cargo test -p agent_context` |
| [x] | P13-T5 | Add live Figma node screenshot fetch API | codex | P13-T1 | `crates/figma_client/src/lib.rs` | `cargo test -p figma_client` | `467d553` | Started 2026-03-05 03:27 CST; completed 2026-03-05 03:28 CST; verified `cargo test -p figma_client` |
| [ ] | P13-T6 | Add `build-agent-context` stage and artifact emission | unassigned | P13-T3,P13-T4 | `crates/orchestrator/Cargo.toml`, `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Emits `output/agent/agent_context.json` + `output/agent/search_index.json` |
| [ ] | P13-T7 | Expose orchestrator lookup tool APIs (`find_nodes`, `get_node_info`) | unassigned | P13-T6 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Explicit statuses: ok/no_match/ambiguous/not_found |
| [ ] | P13-T8 | Add CLI `agent-tool` subcommands in stateless run-and-consume mode | unassigned | P13-T5,P13-T7 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | - | Must not require daemon/session server |
| [ ] | P13-T9 | Add warning + trace artifact writers and playbook docs | unassigned | P13-T8 | `crates/orchestrator/src/lib.rs`, `docs/agent-playbook.md`, `README.md` | `cargo test -p orchestrator && cargo test -p cli` | - | Writes `output/reports/generation_warnings.json` and `output/reports/generation_trace.json` |
| [ ] | P13-T10 | Phase verification, merge to `main`, and merged-main verification | unassigned | P13-T9 | `docs/plans/boards/2026-03-05-phase-13-hybrid-agent-run-and-consume-board.md` | `cargo check --workspace && cargo test --workspace` | - | Update board with pass evidence and merge hash |

## Parallelization Rules

1. `P13-T5` can run in parallel with `P13-T2` to `P13-T4` because files do not overlap.
2. `P13-T6` and `P13-T7` stay sequential because both edit `crates/orchestrator/src/lib.rs`.
3. `P13-T8` and `P13-T9` stay sequential because they share CLI/orchestrator integration points.
4. Do not claim tasks with unmet dependencies.
5. Do not mark a task `[x]` without commit hash and passing verification output.

## Phase Exit Criteria

1. Every task row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. Work is merged into `main`.
5. Verification is re-run on merged `main` and recorded in `P13-T10`.

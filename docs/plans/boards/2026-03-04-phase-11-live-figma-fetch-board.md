# Phase 11 Parallel Board: Live Figma API Fetch

**Phase ID:** `P11`
**Goal:** Add live Figma API/auth fetch support for real end-to-end pipeline runs.
**Source Plan:** `docs/plans/2026-03-04-live-figma-auth-fetch-implementation.md`
**Last Updated:** `2026-03-04 16:32 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [~] | P11-T1 | Add live fetch request/auth contracts | codex | - | `crates/figma_client/src/lib.rs`, `crates/figma_client/Cargo.toml` | `cargo test -p figma_client` | - | Started 2026-03-04 16:32 CST; adding typed live request/auth contracts and explicit fetch error taxonomy |
| [ ] | P11-T2 | Implement live Figma API node fetch | unassigned | P11-T1 | `crates/figma_client/src/lib.rs` | `cargo test -p figma_client` | - | Call `/v1/files/{file_key}/nodes?ids={node_id}` with `X-Figma-Token` |
| [ ] | P11-T3 | Add live transport tests with mock server | unassigned | P11-T2 | `crates/figma_client/Cargo.toml`, `crates/figma_client/src/lib.rs` | `cargo test -p figma_client` | - | Cover success + auth/error mapping without external network |
| [ ] | P11-T4 | Add orchestrator fetch config + live execution | unassigned | P11-T3 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Add config-based stage/run-all entrypoints for fixture vs live |
| [ ] | P11-T5 | Harden orchestrator live fetch actionable errors | unassigned | P11-T4 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | - | Improve guidance for auth/token/file/node failures |
| [ ] | P11-T6 | Expose live fetch options in CLI | unassigned | P11-T5 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | - | Add `--input`, `--file-key`, `--node-id`, `--figma-token` for `fetch`/`generate` |
| [ ] | P11-T7 | Add CLI live-mode validation coverage | unassigned | P11-T6 | `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | - | Validate missing args/env fallback with stable exit code `2` |
| [ ] | P11-T8 | Document live usage in README | unassigned | P11-T6 | `README.md` | docs self-check + command copy/paste validation | - | Add explicit live quickstart and fixture/live mode boundaries |
| [ ] | P11-T9 | Final verification and closeout | unassigned | P11-T7,P11-T8 | `docs/plans/boards/2026-03-04-phase-11-live-figma-fetch-board.md` | `cargo fmt --all --check && cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge phase into `main` and verify again on merged `main` |

## Parallelization Rules

1. `P11-T8` can be prepared in parallel after CLI option names in `P11-T6` are finalized.
2. Core implementation remains mostly sequential because `figma_client` contracts feed orchestrator and CLI.
3. Keep one owner per `[~]` task and avoid concurrent edits to `crates/orchestrator/src/lib.rs` and `crates/cli/src/main.rs`.
4. Mark tasks `[x]` only with commit hash and successful task verification.

## Phase Exit Criteria

1. Every row is `[x]`.
2. `cargo fmt --all --check` passes.
3. `cargo check --workspace` passes.
4. `cargo test --workspace` passes.
5. `bash scripts/verify_workspace.sh` passes.
6. Live CLI path (`fetch`/`generate` with `--input live`) is documented and operable.
7. Phase branch is merged into `main` and verified again on merged `main`.

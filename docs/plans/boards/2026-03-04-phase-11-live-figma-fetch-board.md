# Phase 11 Parallel Board: Live Figma API Fetch

**Phase ID:** `P11`
**Goal:** Add live Figma API/auth fetch support for real end-to-end pipeline runs.
**Source Plan:** `docs/plans/2026-03-04-live-figma-auth-fetch-implementation.md`
**Last Updated:** `2026-03-04 16:49 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P11-T1 | Add live fetch request/auth contracts | codex | - | `crates/figma_client/src/lib.rs`, `crates/figma_client/Cargo.toml` | `cargo test -p figma_client` | `92fab1c` | Started 2026-03-04 16:32 CST; added live request/auth contract types and explicit live fetch error variants |
| [x] | P11-T2 | Implement live Figma API node fetch | codex | P11-T1 | `crates/figma_client/src/lib.rs` | `cargo test -p figma_client` | `d99cf5f` | Started 2026-03-04 16:34 CST; implemented live fetch transport and mapping into canonical raw snapshot payload |
| [x] | P11-T3 | Add live transport tests with mock server | codex | P11-T2 | `crates/figma_client/Cargo.toml`, `crates/figma_client/src/lib.rs` | `cargo test -p figma_client` | `41b4537` | Started 2026-03-04 16:36 CST; added mock HTTP transport tests for auth header, path/query, success, and status mapping |
| [x] | P11-T4 | Add orchestrator fetch config + live execution | codex | P11-T3 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | `85d01e9` | Started 2026-03-04 16:38 CST; added config-based run-stage/run-all entrypoints with live fetch support |
| [x] | P11-T5 | Harden orchestrator live fetch actionable errors | codex | P11-T4 | `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator` | `2563757` | Started 2026-03-04 16:40 CST; added actionable live fetch guidance for token/env, params, and permission failures |
| [x] | P11-T6 | Expose live fetch options in CLI | codex | P11-T5 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | `9476868` | Started 2026-03-04 16:41 CST; added live input flags/config handoff with validation for `fetch`/`generate` |
| [x] | P11-T7 | Add CLI live-mode validation coverage | codex | P11-T6 | `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli` | `12bb949` | Started 2026-03-04 16:45 CST; added env token fallback and deterministic live validation checks |
| [x] | P11-T8 | Document live usage in README | codex | P11-T6 | `README.md` | docs self-check + command copy/paste validation | `2cc26cb` | Started 2026-03-04 16:46 CST; documented fixture vs live start-points and copy/paste commands |
| [x] | P11-T9 | Final verification and closeout | codex | P11-T7,P11-T8 | `docs/plans/boards/2026-03-04-phase-11-live-figma-fetch-board.md` | `cargo fmt --all --check && cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | `e27406e` | Started 2026-03-04 16:47 CST; passed full gates on phase branch, merged into `main`, and re-ran full gates on merged `main` |

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

# Live Figma Auth Fetch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add production-ready live Figma API fetch/auth support so contributors can run the pipeline against a real file/node and inspect generated outputs.

**Architecture:** Keep stage boundaries intact by implementing live fetch in `figma_client`, then pass typed fetch configuration through `orchestrator` into `cli` commands. Preserve deterministic fixture mode as the default fallback for local tests while enabling explicit live mode for real API calls.

**Tech Stack:** Rust 2024 workspace, `reqwest` (blocking client) for HTTPS fetch, `serde`/`serde_json`, `thiserror`, existing `clap` CLI parsing, fixture + mock-server tests.

---

## Outcome and Done Criteria

One-command live run is available and documented:

```bash
cargo run -p cli -- generate --input live --file-key <FILE_KEY> --node-id <NODE_ID> --figma-token "$FIGMA_TOKEN"
```

Done criteria:

1. `fetch` can call live Figma API and persist a valid raw snapshot artifact.
2. `generate` can execute end-to-end from live input without manual artifact editing.
3. Missing/invalid auth and missing live parameters produce actionable, stable error messages.
4. Existing fixture mode and deterministic tests continue to pass.
5. README explains both fixture and live usage.
6. Phase gates pass:
   - `cargo fmt --all --check`
   - `cargo check --workspace`
   - `cargo test --workspace`
   - `bash scripts/verify_workspace.sh`

## Dependency Graph

Sequential constraints:

1. `figma_client` live transport contracts must land before orchestrator can consume them.
2. Orchestrator fetch configuration must land before CLI can expose live options.
3. CLI command surface must land before docs can reference live commands.
4. Final verification/merge runs after all tasks are complete.

Parallelizable work:

1. Core fetch contract unit tests and mock-server tests can run in parallel within the same task scope.
2. README and board docs can be updated after CLI option names are stable.

## Task Plan

### Task P11-T1: Add live fetch request/auth contracts in `figma_client`

**Files:**
- Modify: `crates/figma_client/src/lib.rs`
- Modify: `crates/figma_client/Cargo.toml`
- Test: `crates/figma_client/src/lib.rs`

**Behavior Delta:**
- Introduce typed live fetch inputs (token, API base URL override for tests).
- Add explicit auth/transport/status-code error variants.
- Keep fixture fetch API unchanged for deterministic mode.

**Verification:**
- `cargo test -p figma_client`

**Commit:**
- `feat(figma-client): add live fetch auth and request contracts`

---

### Task P11-T2: Implement live Figma API node fetch transport

**Files:**
- Modify: `crates/figma_client/src/lib.rs`
- Test: `crates/figma_client/src/lib.rs`

**Behavior Delta:**
- Add live API fetch function that calls:
  - `GET /v1/files/{file_key}/nodes?ids={node_id}`
- Send `X-Figma-Token` header.
- Parse live JSON payload into existing `RawFigmaSnapshot` envelope.
- Return explicit errors for unauthorized, not found, and non-success responses.

**Verification:**
- `cargo test -p figma_client`

**Commit:**
- `feat(figma-client): implement live figma api node fetch`

---

### Task P11-T3: Add live transport tests with mock HTTP server

**Files:**
- Modify: `crates/figma_client/Cargo.toml`
- Modify: `crates/figma_client/src/lib.rs`

**Behavior Delta:**
- Add tests that validate:
  - auth header is sent
  - request path/query are correct
  - success payload maps into raw snapshot
  - 401/404/non-JSON map to stable error variants

**Verification:**
- `cargo test -p figma_client`

**Commit:**
- `test(figma-client): cover live fetch transport and auth errors`

---

### Task P11-T4: Add orchestrator fetch config and live execution path

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs`

**Behavior Delta:**
- Add typed fetch config (`fixture` vs `live`) accepted by:
  - `run_stage_with_config(...)`
  - `run_all_with_config(...)`
- Keep existing `run_stage`/`run_all` behavior as fixture default.
- Wire `fetch` stage to call live fetch when config is `live`.

**Verification:**
- `cargo test -p orchestrator`

**Commit:**
- `feat(orchestrator): support live fetch configuration for stage execution`

---

### Task P11-T5: Harden orchestrator actionable errors for live fetch failures

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Test: `crates/orchestrator/src/lib.rs`

**Behavior Delta:**
- Map live auth/fetch failures into actionable guidance (token, file key, node id, permissions).
- Preserve stable `PipelineError` shapes and message prefixes used by CLI tests.

**Verification:**
- `cargo test -p orchestrator`

**Commit:**
- `fix(orchestrator): improve actionable live fetch failure messages`

---

### Task P11-T6: Expose live fetch options in CLI for `fetch` and `generate`

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Behavior Delta:**
- Add CLI options:
  - `--input <fixture|live>`
  - `--file-key <FILE_KEY>`
  - `--node-id <NODE_ID>`
  - `--figma-token <TOKEN>` (fallback to `FIGMA_TOKEN` env)
- Pass parsed config to orchestrator config-based APIs.
- Validate required live args and return exit code `2` with actionable errors.

**Verification:**
- `cargo test -p cli`

**Commit:**
- `feat(cli): add live figma fetch options for fetch and generate`

---

### Task P11-T7: Add CLI live-mode behavior tests (no external network dependency)

**Files:**
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Behavior Delta:**
- Assert required live params produce deterministic errors when missing.
- Assert token env fallback behavior.
- Keep fixture-mode outputs unchanged.

**Verification:**
- `cargo test -p cli`

**Commit:**
- `test(cli): add live input mode validation coverage`

---

### Task P11-T8: Document live usage in README

**Files:**
- Modify: `README.md`

**Behavior Delta:**
- Add explicit live test flow with command examples and required env vars.
- Clarify fixture mode vs live mode start points.

**Verification:**
- Docs self-check with command copy/paste validation

**Commit:**
- `docs(cli): add live figma fetch quickstart`

---

### Task P11-T9: Final phase verification and closeout

**Files:**
- Modify: `docs/plans/boards/2026-03-04-phase-11-live-figma-fetch-board.md`

**Behavior Delta:**
- Mark all board tasks complete with commit hashes.
- Run phase verification gates and merge phase branch to `main`.
- Re-run full verification on merged `main`.

**Verification:**
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `bash scripts/verify_workspace.sh`

**Commit:**
- `chore(workspace): complete phase 11 live fetch verification`

## Phase Exit Criteria

1. Every `P11` board row is `[x]` with commit hash.
2. Live fetch path works from CLI (`fetch` and `generate`) with real parameters.
3. Fixture mode behavior remains backward-compatible.
4. Workspace verification passes before and after merge to `main`.

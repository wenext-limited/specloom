# Figma Hybrid Agent Generation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a strict-hybrid, single-agent generation flow that uses screenshot grounding plus deterministic node lookup tooling with structured warning output.

**Architecture:** Keep `fetch -> normalize -> infer-layout -> build-spec` deterministic, then add a deterministic `build-agent-context` stage that writes JSON contracts for agent startup and fuzzy lookup. Expose tool-style CLI commands (`find-nodes`, `get-node-info`, `get-node-screenshot`) and keep mismatch handling non-fatal by emitting warnings/traces instead of stopping generation. Runtime mode is stateless run-and-consume in v1: commands consume persisted artifacts and exit, with no always-on background process requirement.

**Tech Stack:** Rust 2024 workspace, `serde`, `serde_json`, `ron`, `clap`, `reqwest` (blocking), `thiserror`.

---

Use `@test-driven-development` for each task, `@systematic-debugging` for unexpected failures, and `@verification-before-completion` before merge.

Execution model guardrail:

1. Do not introduce a mandatory daemon/session server in this phase.
2. Keep all agent/tool flows executable via normal CLI invocations that read/write artifacts.

### Task 1: Create `agent_context` Crate and Register It in the Workspace

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent_context/Cargo.toml`
- Create: `crates/agent_context/src/lib.rs`

**Step 1: Write the failing test**

```bash
cargo test -p agent_context
```

Expected: FAIL with unknown package `agent_context`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p agent_context`  
Expected: non-zero exit (`package ID specification 'agent_context' did not match any packages`).

**Step 3: Write minimal implementation**

```toml
# Cargo.toml (workspace)
[workspace]
members = [
  "crates/cli",
  "crates/figma_client",
  "crates/figma_normalizer",
  "crates/layout_infer",
  "crates/ui_spec",
  "crates/asset_pipeline",
  "crates/orchestrator",
  "crates/agent_context",
]
resolver = "2"
```

```toml
# crates/agent_context/Cargo.toml
[package]
name = "agent_context"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
```

```rust
// crates/agent_context/src/lib.rs
#![forbid(unsafe_code)]
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p agent_context`  
Expected: PASS (0 tests initially).

**Step 5: Commit**

```bash
git add Cargo.toml crates/agent_context
git commit -m "chore(workspace): add agent_context crate scaffold"
```

### Task 2: Add JSON Contracts for Agent Context, Search Index, Warnings, and Trace

**Files:**
- Modify: `crates/agent_context/src/lib.rs`
- Test: `crates/agent_context/src/lib.rs`

**Step 1: Write the failing test**

Add tests in `crates/agent_context/src/lib.rs`:

```rust
#[test]
fn agent_context_round_trip_json() {
    let context = AgentContext::sample();
    let encoded = context.to_pretty_json().unwrap();
    let decoded: AgentContext = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, context);
}

#[test]
fn warning_file_round_trip_json() {
    let report = GenerationWarnings::sample();
    let encoded = serde_json::to_vec_pretty(&report).unwrap();
    let decoded: GenerationWarnings = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, report);
}
```

Expected: compile FAIL because these types/APIs do not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p agent_context agent_context_round_trip_json -- --nocapture`  
Expected: compile errors for missing types/functions.

**Step 3: Write minimal implementation**

Add:

1. `AgentContext`, `ScreenRef`, `SkeletonNode`, `GenerationRules`.
2. `SearchIndex`, `SearchIndexEntry`.
3. `GenerationWarnings`, `GenerationWarning`, `GenerationTrace`, `TraceEvent`.
4. `AgentContext::to_pretty_json() -> Result<Vec<u8>, serde_json::Error>`.
5. `#[serde(deny_unknown_fields)]` on persisted contracts.

**Step 4: Run test to verify it passes**

Run: `cargo test -p agent_context`  
Expected: PASS with round-trip tests.

**Step 5: Commit**

```bash
git add crates/agent_context/src/lib.rs
git commit -m "feat(core): add agent context and reporting contracts"
```

### Task 3: Implement Deterministic Fuzzy Search Index and Ranking

**Files:**
- Create: `crates/agent_context/src/search.rs`
- Modify: `crates/agent_context/src/lib.rs`
- Test: `crates/agent_context/src/search.rs`

**Step 1: Write the failing test**

Add tests in `search.rs`:

```rust
#[test]
fn normalize_tokens_lowercases_and_strips_punctuation() {
    assert_eq!(normalize_tokens("Welcome, Back!"), vec!["welcome", "back"]);
}

#[test]
fn rank_candidates_is_stable_with_tie_break_on_node_id() {
    let results = rank_candidates("title", sample_entries(), 5);
    assert_eq!(results[0].node_id, "1:10");
    assert_eq!(results[1].node_id, "1:11");
}

#[test]
fn rank_candidates_marks_low_confidence_and_no_match_thresholds() {
    let status = classify_status(0.50);
    assert_eq!(status, SearchStatus::LowConfidence);
}
```

Expected: compile FAIL because ranking APIs are missing.

**Step 2: Run test to verify it fails**

Run: `cargo test -p agent_context rank_candidates_is_stable_with_tie_break_on_node_id -- --nocapture`  
Expected: compile errors.

**Step 3: Write minimal implementation**

Implement:

1. `normalize_tokens(input: &str) -> Vec<String>`.
2. `rank_candidates(query: &str, entries: &[SearchIndexEntry], top_k: usize)`.
3. Weighted scoring:
   - token overlap `0.45`
   - alias match `0.20`
   - path similarity `0.20`
   - geometry hint `0.15`
4. Deterministic sort: `(score desc, node_id asc)`.
5. Threshold classification:
   - `>= 0.72` confident
   - `0.45..0.72` low confidence
   - `< 0.45` no match

**Step 4: Run test to verify it passes**

Run: `cargo test -p agent_context`  
Expected: PASS with deterministic ranking tests.

**Step 5: Commit**

```bash
git add crates/agent_context/src/lib.rs crates/agent_context/src/search.rs
git commit -m "feat(core): add deterministic fuzzy node ranking"
```

### Task 4: Add Figma Node Screenshot Fetch API in `figma_client`

**Files:**
- Modify: `crates/figma_client/src/lib.rs`
- Test: `crates/figma_client/src/lib.rs`

**Step 1: Write the failing test**

Add tests for screenshot endpoint behavior:

```rust
#[test]
fn fetch_node_screenshot_live_requests_images_endpoint() {
    // mock server verifies:
    // GET /v1/images/<file_key>?ids=<node_id>&format=png
}

#[test]
fn fetch_node_screenshot_live_reports_missing_image_ref() {
    // API payload without images[node_id] returns InvalidApiResponse
}
```

Expected: compile FAIL because screenshot API is not implemented.

**Step 2: Run test to verify it fails**

Run: `cargo test -p figma_client fetch_node_screenshot_live_requests_images_endpoint -- --nocapture`  
Expected: compile errors for missing request/response APIs.

**Step 3: Write minimal implementation**

Add:

1. `LiveScreenshotRequest` (`file_key`, `node_id`, `figma_token`, `api_base_url`).
2. `NodeScreenshot` contract (`node_id`, `image_url`, `format`).
3. `fetch_node_screenshot_live()` using `GET /v1/images/{file_key}` with `ids` and `format=png`.
4. Explicit errors for unauthorized, non-success status, and missing image refs.

**Step 4: Run test to verify it passes**

Run: `cargo test -p figma_client`  
Expected: PASS, including existing fetch tests.

**Step 5: Commit**

```bash
git add crates/figma_client/src/lib.rs
git commit -m "feat(figma-client): add live node screenshot fetch API"
```

### Task 5: Add `build-agent-context` Stage in Orchestrator

**Files:**
- Modify: `crates/orchestrator/Cargo.toml`
- Modify: `crates/orchestrator/src/lib.rs`

**Step 1: Write the failing test**

Add orchestrator test:

```rust
#[test]
fn run_stage_build_agent_context_writes_agent_artifacts() {
    let root = unique_test_workspace_root("run_stage_build_agent_context");
    run_stage_in_workspace("fetch", root.as_path()).unwrap();
    run_stage_in_workspace("normalize", root.as_path()).unwrap();
    run_stage_in_workspace("infer-layout", root.as_path()).unwrap();
    run_stage_in_workspace("build-spec", root.as_path()).unwrap();
    let result = run_stage_in_workspace("build-agent-context", root.as_path()).unwrap();
    assert_eq!(result.artifact_path, Some("output/agent/agent_context.json".to_string()));
    assert!(root.join("output/agent/search_index.json").is_file());
}
```

Expected: FAIL with unknown stage.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator run_stage_build_agent_context_writes_agent_artifacts -- --nocapture`  
Expected: unknown stage failure.

**Step 3: Write minimal implementation**

Implement:

1. Add `agent_context` dependency.
2. Stage constants:
   - `AGENT_CONTEXT_ARTIFACT_RELATIVE_PATH = "output/agent/agent_context.json"`
   - `SEARCH_INDEX_ARTIFACT_RELATIVE_PATH = "output/agent/search_index.json"`
3. Add stage definition and `run_all` ordering:
   - `fetch -> normalize -> infer-layout -> build-spec -> build-agent-context -> export-assets`
4. Parse `output/specs/ui_spec.ron`, project skeleton, build search index, write JSON artifacts.

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator`  
Expected: PASS, including stage order assertions.

**Step 5: Commit**

```bash
git add crates/orchestrator/Cargo.toml crates/orchestrator/src/lib.rs
git commit -m "feat(orchestrator): add build-agent-context stage"
```

### Task 6: Expose Orchestrator Tool Functions (`find_nodes`, `get_node_info`)

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`

**Step 1: Write the failing test**

Add tests:

```rust
#[test]
fn find_nodes_in_workspace_returns_ranked_candidates() {
    // runs build-agent-context then queries "welcome back"
    // expects sorted candidates with scores and reasons
}

#[test]
fn get_node_info_in_workspace_returns_not_found_for_missing_node() {
    // expects explicit not_found status, not panic
}
```

Expected: compile FAIL because public tool functions/types are missing.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator find_nodes_in_workspace_returns_ranked_candidates -- --nocapture`  
Expected: compile errors.

**Step 3: Write minimal implementation**

Add public APIs:

1. `find_nodes_in_workspace(workspace_root, query, top_k) -> FindNodesResult`.
2. `get_node_info_in_workspace(workspace_root, node_id) -> NodeInfoResult`.
3. Explicit status enums:
   - `FindNodesStatus`: `Ok | NoMatch | Ambiguous`
   - `NodeInfoStatus`: `Ok | NotFound`

Load from deterministic artifacts (`search_index.json`, normalized payload), never from ad-hoc in-memory state.

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator`  
Expected: PASS with new tool-api tests.

**Step 5: Commit**

```bash
git add crates/orchestrator/src/lib.rs
git commit -m "feat(orchestrator): add deterministic node lookup tool APIs"
```

### Task 7: Add CLI Agent Tool Commands

**Files:**
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`

**Step 1: Write the failing test**

Add command tests:

```rust
#[test]
fn agent_tool_find_nodes_json_mode_returns_candidates() {
    // cli agent-tool find-nodes --query "welcome" --output json
}

#[test]
fn agent_tool_get_node_info_reports_not_found_actionably() {
    // cli agent-tool get-node-info --node-id missing
}
```

Expected: FAIL because subcommands do not exist.

**Step 2: Run test to verify it fails**

Run: `cargo test -p cli --test commands agent_tool_find_nodes_json_mode_returns_candidates -- --nocapture`  
Expected: clap unknown subcommand assertion failures.

**Step 3: Write minimal implementation**

Add CLI surface:

1. `agent-tool find-nodes --query <text> [--top-k <n>] [--output text|json]`
2. `agent-tool get-node-info --node-id <id> [--output text|json]`
3. `agent-tool get-node-screenshot --file-key <key> --node-id <id> [--figma-token <token>] [--figma-api-base-url <url>] [--output text|json]`

Wire first two via orchestrator APIs and screenshot command via `figma_client`.
Do not add a required persistent service for these commands; they must run in a single invocation and exit.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cli`  
Expected: PASS with updated command and smoke coverage.

**Step 5: Commit**

```bash
git add crates/cli/src/main.rs crates/cli/tests/commands.rs crates/cli/tests/integration_smoke.rs
git commit -m "feat(cli): add agent-tool lookup and screenshot commands"
```

### Task 8: Emit Generation Warning/Trace Artifacts and Add Agent Playbook Doc

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Create: `docs/agent-playbook.md`
- Modify: `README.md`

**Step 1: Write the failing test**

Add orchestrator test:

```rust
#[test]
fn tool_lookup_no_match_emits_warning_artifact_entry() {
    // simulate a no-match lookup and assert warning file includes NODE_NOT_FOUND
}
```

Expected: FAIL because warning writer is not implemented.

**Step 2: Run test to verify it fails**

Run: `cargo test -p orchestrator tool_lookup_no_match_emits_warning_artifact_entry -- --nocapture`  
Expected: compile/runtime failure for missing file writer logic.

**Step 3: Write minimal implementation**

Implement:

1. Warning file writer at `output/reports/generation_warnings.json`.
2. Trace writer at `output/reports/generation_trace.json`.
3. Append warning entries from lookup statuses (`NO_MATCH`, `LOW_CONFIDENCE`, `AMBIGUOUS`).
4. Document agent tool sequence and policy in `docs/agent-playbook.md`.
5. Update `README.md` with new stage and command matrix rows.

**Step 4: Run test to verify it passes**

Run: `cargo test -p orchestrator && cargo test -p cli`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/orchestrator/src/lib.rs docs/agent-playbook.md README.md
git commit -m "docs(workspace): add agent playbook and warning trace workflow"
```

### Task 9: Final Verification Gate

**Files:**
- Verify only

**Step 1: Run full workspace checks**

```bash
cargo check --workspace
cargo test --workspace
```

Expected: PASS.

**Step 2: Commit any final fixture/doc updates**

If test fixtures changed:

```bash
git add crates/cli/tests/fixtures/generate_expected_output.json
git commit -m "test(cli): refresh generate fixture for agent context stage"
```

**Step 3: Record verification evidence in handoff**

Include exact command list and final pass status in the PR or handoff summary.

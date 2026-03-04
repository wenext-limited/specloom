# Agent-Driven Build-Spec Transform Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `infer-layout` with an agent-driven transform flow where the agent decides high-level UI node types and child handling via a structured transform plan.

**Architecture:** Keep deterministic `fetch -> normalize`, then in `build-spec` generate `pre_layout.ron` and `node_map.json`, call agent to produce `transform_plan.json`, and apply that plan mechanically (no post-AI semantic re-inference). Emit `ui_spec.ron` as final output, then run `build-agent-context` from final spec.

**Tech Stack:** Rust 2024 workspace, `serde`, `serde_json`, `ron`, `clap`, existing `llm_codegen`/agent command plumbing.

---

Use `@test-driven-development` for each task, `@systematic-debugging` for unexpected command output, and `@verification-before-completion` before phase close.

Execution guardrails:

1. Agent decides `suggested_type` and `child_policy`.
2. Rust validates schema/references and applies transforms mechanically.
3. Do not add fallback semantic inference after AI output.

## Contracts to Add

### `output/specs/pre_layout.ron`

Initial pre-processed Rust object tree (before agent transforms).

### `output/specs/node_map.json`

Deterministic map:

```json
{
  "version": "node_map/1.0",
  "nodes": {
    "1:10": { "...raw normalized node payload..." }
  }
}
```

### `output/specs/transform_plan.json`

Agent result contract:

```json
{
  "version": "transform_plan/1.0",
  "decisions": [
    {
      "node_id": "1:10",
      "suggested_type": "Button",
      "child_policy": {
        "mode": "drop"
      },
      "confidence": 0.82,
      "reason": "Container is action-like with label/icon composition"
    }
  ]
}
```

`child_policy.mode` allowed values:

1. `keep`
2. `drop`
3. `replace_with`

If `replace_with`, include `children: ["<id>", ...]`.

## Task 1: Remove `infer-layout` from the Active Orchestration Path

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `crates/cli/tests/commands.rs`
- Modify: `crates/cli/tests/integration_smoke.rs`
- Modify: `crates/cli/tests/fixtures/generate_expected_output.json`

**Steps:**
1. Write failing tests expecting stage order without `infer-layout`.
2. Run failing tests.
3. Remove `infer-layout` from stage list and default `generate` order.
4. Keep compatibility decision explicit:
   - either remove `run-stage infer-layout`, or
   - keep it as deprecated/manual but not in default flow.
5. Re-run CLI/orchestrator tests.
6. Commit.

## Task 2: Add Pre-Layout and Node-Map Artifacts in `build-spec`

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `crates/ui_spec/src/build.rs`
- Add tests in affected crates

**Steps:**
1. Add failing tests for artifact presence:
   - `output/specs/pre_layout.ron`
   - `output/specs/node_map.json`
2. Implement deterministic serialization in `build-spec`.
3. Ensure stable key ordering in `node_map.json`.
4. Re-run tests.
5. Commit.

## Task 3: Define Transform Plan Contract and Validation

**Files:**
- Create: `crates/ui_spec/src/transform_plan.rs`
- Modify: `crates/ui_spec/src/lib.rs`
- Add tests in `crates/ui_spec/src/tests.rs`

**Steps:**
1. Add failing contract round-trip tests.
2. Add transform plan structs and `child_policy` enum.
3. Add validation:
   - node IDs must exist
   - `replace_with` children must exist
   - allowed `suggested_type` values must parse
4. Re-run tests.
5. Commit.

## Task 4: Apply Transform Plan Mechanically (No Post-AI Inference)

**Files:**
- Modify: `crates/ui_spec/src/build.rs`
- Add tests in `crates/ui_spec/src/tests.rs`

**Steps:**
1. Add failing tests for:
   - `drop` removes children
   - `keep` preserves children
   - `replace_with` rewires children
   - suggested high-level type mapping (`Button`, `ScrollView`, `HStack`, etc.)
2. Implement transform application directly from plan decisions.
3. Ensure no extra semantic fallback heuristics after transform.
4. Re-run tests.
5. Commit.

## Task 5: Wire Agent Invocation for Transform Plan Production

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/tests/commands.rs`

**Steps:**
1. Add failing test for build-spec expecting transform plan file.
2. Invoke agent path (existing model plumbing) with `pre_layout.ron` + `node_map.json`.
3. Persist `transform_plan.json`.
4. Apply transform and output final `ui_spec.ron`.
5. Re-run CLI/orchestrator tests.
6. Commit.

## Task 6: Regenerate Agent Context from Final Spec and Update Docs

**Files:**
- Modify: `crates/orchestrator/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/agent-playbook.md`

**Steps:**
1. Add failing test to verify `build-agent-context` reads final `ui_spec.ron`.
2. Confirm generated context reflects transformed tree shape.
3. Update docs to explain:
   - pre-layout
   - node map
   - transform plan
   - child policy semantics
4. Re-run tests.
5. Commit.

## Task 7: Phase Verification

**Verification:**

```bash
cargo check --workspace
cargo test --workspace
```

Record evidence in board close-out task and merge to `main`.

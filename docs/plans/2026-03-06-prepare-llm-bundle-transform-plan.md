# Prepare LLM Bundle Transform Plan Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `prepare-llm-bundle` guarantee a non-empty, applied transform plan before it writes the bundle.

**Architecture:** Add a transform-readiness step inside core bundle preparation. Reuse existing authored plans when valid, heuristically author a plan when the current one is missing or empty, then refresh `build-spec` and `build-agent-context` before bundling.

**Tech Stack:** Rust 2024 workspace, `specloom-core`, existing `ui_spec` transform contracts, CLI/core tests.

---

### Task 1: Add failing transform-readiness tests

**Files:**
- Modify: `crates/core/src/tests.rs`

**Step 1: Write the failing tests**

Add tests covering:

1. `prepare_llm_bundle` authors a non-empty plan when the existing plan is empty.
2. `prepare_llm_bundle` preserves an existing non-empty plan.

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core prepare_llm_bundle_authors_transform_plan_when_existing_plan_is_empty -- --nocapture`

**Step 3: Implement minimal support**

Add test helpers for a transformable snapshot pipeline.

**Step 4: Run test to verify it passes**

Run the same targeted tests again.

### Task 2: Add transform-readiness gate inside bundle preparation

**Files:**
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/src/ui_spec.rs`

**Step 1: Add readiness helpers**

Implement helpers to:

1. Read optional transform plans.
2. Author a non-empty heuristic plan when missing/empty.
3. Re-run `build-spec`.
4. Re-run `build-agent-context`.

**Step 2: Validate behavior**

1. Preserve non-empty valid plans.
2. Reject invalid non-empty plans.
3. Ensure refreshed bundle artifacts point at transformed outputs.

**Step 3: Re-run focused tests**

Run:

1. `cargo test -p specloom-core prepare_llm_bundle_authors_transform_plan_when_existing_plan_is_empty -- --nocapture`
2. `cargo test -p specloom-core prepare_llm_bundle_reuses_existing_non_empty_transform_plan -- --nocapture`

### Task 3: Update workflow documentation

**Files:**
- Modify: `docs/proposal.md`
- Modify: `docs/agent-playbook.md`
- Modify: `docs/figma-ui-coder.md`

**Step 1: Update current workflow wording**

Reflect that `prepare-llm-bundle` now guarantees transform readiness and refreshed downstream context.

**Step 2: Verify docs match behavior**

Use `rg -n "transform plan|prepare-llm-bundle" docs/proposal.md docs/agent-playbook.md docs/figma-ui-coder.md`

### Task 4: Verify workspace

**Files:**
- None

**Step 1: Run repo verification**

Run:

1. `cargo check --workspace`
2. `cargo test --workspace`

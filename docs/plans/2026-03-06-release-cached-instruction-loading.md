# Release-Cached Instruction Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace per-file raw GitHub instruction fetching with tagged release snapshot caching under `~/.config/specloom`, with fallback to the latest release when the running CLI version is not yet released.

**Architecture:** Keep workspace-local instruction files authoritative. When local files are missing, resolve a release tag, download one snapshot archive, extract it into `~/.config/specloom/release_cache/<tag>/`, then load `.codex/SKILLS.md`, referenced skill docs, and agent playbooks from that extracted snapshot. Preserve deterministic cache reuse and explicit failure messages.

**Tech Stack:** Rust 2024 workspace, `reqwest` blocking client, `serde_json`, archive extraction crates, existing `specloom-core` bundle preparation/tests.

---

Use `@test-driven-development` for each behavior change, `@systematic-debugging` for unexpected failures, and `@verification-before-completion` before claiming completion.

### Task 1: Replace Remote Per-File Fetch Test Coverage

**Files:**
- Modify: `crates/core/src/tests.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write the failing test**

Add a test that seeds a fixture pipeline with no local instruction files, serves:

1. release metadata for `v<CARGO_PKG_VERSION>`
2. a tagged archive containing `.codex/SKILLS.md`, `docs/agent-playbook.md`, `docs/figma-ui-coder.md`, and referenced skill docs

Assert that `prepare_llm_bundle_in_workspace_with_instruction_overrides(...)` succeeds and writes the extracted files under `release_cache/<tag>/`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core prepare_llm_bundle_downloads_tagged_release_snapshot_when_local_files_are_missing -- --nocapture`
Expected: FAIL because the current implementation only fetches per-file raw URLs and uses `skills_cache`.

**Step 3: Write minimal implementation**

Update the instruction-loading pipeline to resolve and cache release snapshots instead of fetching each file separately.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core prepare_llm_bundle_downloads_tagged_release_snapshot_when_local_files_are_missing -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/tests.rs crates/core/src/lib.rs crates/core/Cargo.toml
git commit -m "fix(core): cache bundle instructions from release snapshots"
```

### Task 2: Add Latest-Release Fallback Coverage

**Files:**
- Modify: `crates/core/src/tests.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write the failing test**

Add a test where:

1. `v<CARGO_PKG_VERSION>` and `<CARGO_PKG_VERSION>` release lookups return not found
2. latest-release metadata returns a different tag
3. the archive for that latest tag contains the required instruction files

Assert that bundle preparation succeeds and the cache is created under `release_cache/<latest-tag>/`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p specloom-core prepare_llm_bundle_falls_back_to_latest_release_snapshot_when_current_version_is_unreleased -- --nocapture`
Expected: FAIL because there is no latest-release metadata fallback today.

**Step 3: Write minimal implementation**

Teach instruction resolution to query latest-release metadata only after current-version tags fail.

**Step 4: Run test to verify it passes**

Run: `cargo test -p specloom-core prepare_llm_bundle_falls_back_to_latest_release_snapshot_when_current_version_is_unreleased -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/tests.rs crates/core/src/lib.rs
git commit -m "feat(core): add latest release fallback for bundle instructions"
```

### Task 3: Preserve Local-First and Cache-First Behavior

**Files:**
- Modify: `crates/core/src/tests.rs`
- Modify: `crates/core/src/lib.rs`
- Test: `crates/core/src/tests.rs`

**Step 1: Write or update the failing tests**

Adjust existing tests so they assert:

1. workspace-local files are preferred over any release cache
2. an existing extracted `release_cache/<tag>/` snapshot is used without network requests

**Step 2: Run tests to verify they fail**

Run: `cargo test -p specloom-core prepare_llm_bundle_prefers_local_instruction_files_when_available prepare_llm_bundle_uses_cached_release_snapshot_when_available -- --nocapture`
Expected: FAIL until the cache layout and resolver are updated.

**Step 3: Write minimal implementation**

Refactor helper functions to resolve instruction text from:

1. workspace
2. extracted release cache
3. downloaded snapshot

**Step 4: Run tests to verify they pass**

Run: `cargo test -p specloom-core prepare_llm_bundle_prefers_local_instruction_files_when_available prepare_llm_bundle_uses_cached_release_snapshot_when_available -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/core/src/tests.rs crates/core/src/lib.rs
git commit -m "refactor(core): resolve bundle instructions from local and cached snapshots"
```

### Task 4: Update User-Facing Docs

**Files:**
- Modify: `README.md`
- Modify: `crates/cli/README.md`
- Modify: `docs/plans/2026-03-06-release-cached-instruction-loading-design.md`

**Step 1: Write the doc changes**

Describe:

1. local-first instruction resolution
2. release snapshot cache location under `~/.config/specloom/release_cache/<tag>/`
3. latest-release fallback when the running version has no matching release

**Step 2: Run consistency checks**

Run: `rg -n "release_cache|latest release|prepare-llm-bundle" README.md crates/cli/README.md docs/plans/2026-03-06-release-cached-instruction-loading-design.md`
Expected: PASS with updated wording.

**Step 3: Commit**

```bash
git add README.md crates/cli/README.md docs/plans/2026-03-06-release-cached-instruction-loading-design.md docs/plans/2026-03-06-release-cached-instruction-loading.md
git commit -m "docs(workspace): document release-cached bundle instructions"
```

### Task 5: Full Verification

**Files:**
- Modify: any files changed above as needed

**Step 1: Run formatting and verification**

Run: `cargo fmt --all`
Run: `cargo check --workspace`
Run: `cargo test --workspace`

Expected: PASS.

**Step 2: Summarize verification evidence**

Record the exact commands and outcomes in the handoff.

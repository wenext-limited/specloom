# Phase 12 Parallel Board: UI Blueprint and LLM Generation

**Phase ID:** `P12`
**Goal:** Add a deterministic `ui_blueprint.yaml` artifact and LLM-oriented UI generation tooling while preserving deterministic Rust pipeline stages.
**Source Plan:** `docs/plans/2026-03-04-ui-blueprint-llm-generation.md`
**Last Updated:** `2026-03-04 18:07 CST`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [x] | P12-T1 | Create `ui_blueprint` crate with core YAML contract types | codex | - | `Cargo.toml`, `crates/ui_blueprint/Cargo.toml`, `crates/ui_blueprint/src/lib.rs` | `cargo test -p ui_blueprint` | `e55c965` | Started 2026-03-04 17:55 CST; added core contract types and round-trip/default-version tests; verified with `cargo test -p ui_blueprint` |
| [x] | P12-T2 | Implement `ui_spec -> ui_blueprint` mapper with semantic layout names | codex | P12-T1 | `crates/ui_blueprint/Cargo.toml`, `crates/ui_blueprint/src/lib.rs` | `cargo test -p ui_blueprint build_blueprint_from_ui_spec_maps_root_layout_and_warnings -- --nocapture && cargo test -p ui_blueprint` | `641f9d2` | Started 2026-03-04 17:57 CST; added `build_ui_blueprint` conversion and semantic layout mapping (`stack_v`, `stack_h`, `overlay`, `absolute`, `scroll`) with passing mapper test |
| [x] | P12-T3 | Add deterministic YAML serialization API and stability tests | codex | P12-T2 | `crates/ui_blueprint/src/lib.rs` | `cargo test -p ui_blueprint to_yaml_is_stable_for_identical_blueprint -- --nocapture && cargo test -p ui_blueprint` | `bb70e86` | Started 2026-03-04 17:59 CST; added `to_yaml_string` API and deterministic YAML stability test with passing verification |
| [x] | P12-T4 | Add `build-ui-blueprint` stage in orchestrator and default run order | codex | P12-T3 | `crates/orchestrator/Cargo.toml`, `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator run_stage_build_ui_blueprint_writes_yaml_artifact -- --nocapture && cargo test -p orchestrator` | `4c9a580` | Started 2026-03-04 18:00 CST; added new stage, blueprint artifact output, and default `run_all` order without `gen-swiftui` |
| [x] | P12-T5 | Expose Blueprint-first flow in CLI (`build-ui-blueprint` + generate output updates) | codex | P12-T4 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/e2e_generate.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli --test integration_smoke generate_success_smoke -- --nocapture && cargo test -p cli` | `2e7c6bc` | Started 2026-03-04 18:03 CST; added CLI support for `build-ui-blueprint`, hid legacy `gen-swiftui`, and updated generate/stages expectations |
| [x] | P12-T6 | Create `llm_bundle` crate for deterministic bundle metadata and hashes | codex | P12-T3 | `Cargo.toml`, `crates/llm_bundle/Cargo.toml`, `crates/llm_bundle/src/lib.rs`, `output/llm/.gitkeep` | `cargo test -p llm_bundle` | `d6ef85d` | Started 2026-03-04 18:01 CST; added deterministic bundle/hash crate, warning summaries, and passing tests |
| [x] | P12-T7 | Add orchestrator `prepare-llm-bundle` stage | codex | P12-T4,P12-T6 | `crates/orchestrator/Cargo.toml`, `crates/orchestrator/src/lib.rs` | `cargo test -p orchestrator prepare_llm_bundle -- --nocapture && cargo test -p orchestrator` | `97a742a` | Started 2026-03-04 18:05 CST; added stage execution, warning summary extraction, deterministic bundle artifact writing, and passing tests |
| [~] | P12-T8 | Add CLI support/tests for `prepare-llm-bundle` | codex | P12-T7 | `crates/cli/src/main.rs`, `crates/cli/tests/commands.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli --test integration_smoke prepare_llm_bundle_success_smoke -- --nocapture && cargo test -p cli` | - | Started 2026-03-04 18:07 CST; adding CLI command and smoke coverage for `prepare-llm-bundle` |
| [ ] | P12-T9 | Create `llm_codegen` crate for direct model-based UI generation and run records | unassigned | P12-T6 | `crates/llm_codegen/Cargo.toml`, `crates/llm_codegen/src/lib.rs` | `cargo test -p llm_codegen` | - | Include request validation, mock-server parsing, file write tests |
| [ ] | P12-T10 | Wire `generate-ui` command through orchestrator + CLI | unassigned | P12-T7,P12-T8,P12-T9 | `crates/orchestrator/Cargo.toml`, `crates/orchestrator/src/lib.rs`, `crates/cli/src/main.rs`, `crates/cli/tests/integration_smoke.rs` | `cargo test -p cli --test integration_smoke generate_ui_requires_api_key -- --nocapture && cargo test -p orchestrator && cargo test -p cli` | - | Resolve API key from `--api-key` or `OPENAI_API_KEY` |
| [ ] | P12-T11 | Update docs for UI Blueprint and LLM workflow | unassigned | P12-T5,P12-T8,P12-T10 | `README.md`, `docs/proposal.md`, `docs/plans/2026-03-04-figma-swiftui-generator.md` | `rg -n "ui_blueprint.yaml|build-ui-blueprint|prepare-llm-bundle|generate-ui" README.md docs/proposal.md docs/plans/2026-03-04-figma-swiftui-generator.md` | - | Keep docs aligned with new default stage order |
| [ ] | P12-T12 | Phase verification, merge to `main`, and merged-main re-verification | unassigned | P12-T11 | `docs/plans/boards/2026-03-04-phase-12-ui-blueprint-llm-board.md` | `cargo check --workspace && cargo test --workspace && bash scripts/verify_workspace.sh` | - | Merge completed phase into `main`, then re-run full verification on merged `main` |

## Parallelization Rules

1. `P12-T4` and `P12-T6` can run in parallel after `P12-T3` because files do not overlap.
2. `P12-T7` and `P12-T9` can run in parallel after their dependencies are met.
3. CLI-heavy tasks `P12-T5`, `P12-T8`, and `P12-T10` must run sequentially because they share `crates/cli/src/main.rs` and `crates/cli/tests/integration_smoke.rs`.
4. Orchestrator-heavy tasks `P12-T4`, `P12-T7`, and `P12-T10` must not be claimed concurrently.
5. Do not mark any row `[x]` without task-level verification output and a commit hash.

## Phase Exit Criteria

1. Every task row is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. `bash scripts/verify_workspace.sh` passes.
5. Phase branch is merged into `main`.
6. Full verification is re-run on merged `main` and recorded in `P12-T12` notes.

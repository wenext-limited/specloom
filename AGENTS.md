# AGENTS.md

## Purpose

This document defines how coding agents should operate in this repository.
Follow these rules for all changes unless the user explicitly overrides them.

## Project Context

This is a Rust 2024 workspace for a Figma node tree to SwiftUI pipeline.
Current crate layout:

- `crates/cli`
- `crates/figma_client`
- `crates/figma_normalizer`
- `crates/layout_infer`
- `crates/ui_spec`
- `crates/swiftui_ast`
- `crates/swiftui_codegen`
- `crates/asset_pipeline`
- `crates/review_report`
- `crates/orchestrator`

Key docs:

- `docs/proposal.md`
- `docs/plans/2026-03-04-figma-swiftui-generator-design.md`
- `docs/plans/2026-03-04-figma-swiftui-generator.md`
- `docs/commit-style.md`
- `docs/plans/boards/README.md`
- `docs/plans/templates/parallel-phase-board-template.md`
- `.codex/SKILLS.md`

## Agent Workflow

1. Read `docs/proposal.md` and relevant plan docs before implementing.
2. Keep changes small, explicit, and scoped to one goal.
3. Prefer deterministic behavior and stable serialization/output.
4. Never silently drop unsupported features; surface warnings clearly.
5. Add tests with behavior changes whenever practical.
6. Verify before claiming completion.
7. For parallelizable phases, use a board in `docs/plans/boards/` with task statuses:
   - `[ ]` not started
   - `[~]` in progress
   - `[x]` completed

## Phase Transition Rule

1. Treat each milestone/phase as an isolated branch of work.
2. When a phase is complete and verified, merge it into `main` locally unless the user asks otherwise.
3. After merging, verify again on merged `main`.
4. Start the next phase from a new branch based on the latest `main` (not from the previous phase branch).
5. Commit after each completed task within a phase.

## Coding Rules

1. Use Rust 2024 (`edition = "2024"`).
2. Preserve crate boundaries by concern; avoid random cross-crate coupling.
3. Keep public contracts versioned where persisted artifacts are involved.
4. Prefer explicit errors over implicit fallback behavior.
5. Avoid hidden side effects in stage orchestration.

## Safety Rules

1. Do not revert or delete unrelated user changes.
2. Avoid destructive git commands unless explicitly requested.
3. Do not introduce credentials or secrets into code or logs.
4. Keep generated/build artifacts out of commits.

## Verification Rules

Run these before claiming success for code changes:

```bash
cargo check --workspace
cargo test --workspace
```

For docs-only updates, provide a brief self-check summary instead.

## Commit and PR Rules

Use the commit convention defined in:

- `docs/commit-style.md`

Include verification evidence in your handoff summary or PR description.

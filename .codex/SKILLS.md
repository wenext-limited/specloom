# Project Skills Guide

This file defines the project-local skills and how agents should use them.

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use when deciding container layout strategy (`v_stack`, `h_stack`, `overlay`, `absolute`, `scroll`) from node geometry and metadata.

2. `generating-ui-spec-ron`
Path: `.codex/skills/generating-ui-spec-ron/SKILL.md`
Use when producing final `output/specs/ui_spec.ron` from `pre_layout.ron`, `node_map.json`, and screenshot grounding through `transform_plan.json`.

3. `generating-ui-from-ui-spec-ron`
Path: `.codex/skills/generating-ui-from-ui-spec-ron/SKILL.md`
Use when `ui_spec.ron` is ready and the agent must generate UI code that matches the user-requested target and constraints.

4. `planning-implementation-work`
Path: `.codex/skills/planning-implementation-work/SKILL.md`
Use when converting approved scope into a phased implementation plan with dependencies, verification gates, and commit boundaries.

5. `parallel-phase-workflow`
Path: `.codex/skills/parallel-phase-workflow/SKILL.md`
Use when executing or maintaining phase boards with `[ ]`, `[~]`, `[x]` status and merge-to-main transitions.

## Usage Order

1. Run `recognizing-layout` first when layout decisions are needed.
2. Run `generating-ui-spec-ron` when converting pre-layout artifacts into final `ui_spec.ron`.
3. Run `generating-ui-from-ui-spec-ron` after `ui_spec.ron` exists and generation target is known.
4. Run `planning-implementation-work` after scope is approved.
5. Run `parallel-phase-workflow` while executing plan boards.

## Prompt Patterns

- `Use $recognizing-layout to classify layout for <node set> and output confidence + warnings.`
- `Use $generating-ui-spec-ron to produce transform_plan.json and final ui_spec.ron from pre_layout.ron + node_map.json + screenshot.`
- `Use $generating-ui-from-ui-spec-ron to generate UI code from ui_spec.ron in the exact target requested by the user.`
- `Use $planning-implementation-work to produce a phased plan and boards for <approved scope>.`
- `Use $parallel-phase-workflow to execute the active board and update task status with verification evidence.`

## Rules

1. Do not mix layout inference and implementation planning in one skill.
2. Keep warnings explicit for low-confidence or unsupported cases.
3. Never shortcut `ui_spec.ron` by copying `pre_layout.ron`; always apply `transform_plan.json` through `build-spec`.
4. Every planned task must include a verification command.

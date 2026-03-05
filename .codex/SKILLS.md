# Project Skills Guide

This file defines project-local skills and recommended usage order.

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use when deciding container layout strategy (`v_stack`, `h_stack`, `overlay`, `absolute`, `scroll`) from node geometry and metadata.

2. `generating-ui-spec-ron`
Path: `.codex/skills/generating-ui-spec-ron/SKILL.md`
Use when orchestrating screenshot-grounded pre-layout artifacts into final `output/specs/ui_spec.ron`.

3. `node-grounding-for-transform`
Path: `.codex/skills/node-grounding-for-transform/SKILL.md`
Use when transform decisions need authoritative node evidence plus screenshot grounding for root/ambiguous nodes.

4. `authoring-transform-plan`
Path: `.codex/skills/authoring-transform-plan/SKILL.md`
Use when writing/validating `output/specs/transform_plan.json` decisions.

5. `simplifying-system-components`
Path: `.codex/skills/simplifying-system-components/SKILL.md`
Use when system chrome should be flattened to concise semantic structure.

6. `inferring-repeat-element-ids`
Path: `.codex/skills/inferring-repeat-element-ids/SKILL.md`
Use when deciding whether repeated-node metadata should be encoded.

7. `generating-ui-from-ui-spec-ron`
Path: `.codex/skills/generating-ui-from-ui-spec-ron/SKILL.md`
Use when final `ui_spec.ron` exists and target UI code must be generated.

8. `planning-implementation-work`
Path: `.codex/skills/planning-implementation-work/SKILL.md`
Use when converting approved scope into a phased implementation plan.

9. `parallel-phase-workflow`
Path: `.codex/skills/parallel-phase-workflow/SKILL.md`
Use when executing or maintaining dependency-aware phase boards.

## Usage Order

1. Run `recognizing-layout` when layout decisions are needed.
2. Run `generating-ui-spec-ron` for pre-layout -> final spec orchestration.
3. Inside that flow, use:
   - `node-grounding-for-transform`
   - `simplifying-system-components` (if needed)
   - `inferring-repeat-element-ids` (if needed)
   - `authoring-transform-plan`
4. Run `generating-ui-from-ui-spec-ron` after final spec exists.
5. Use planning/phase skills for implementation execution.

## Rules

1. Never shortcut `ui_spec.ron` by copying `pre_layout.ron`.
2. Always apply transform decisions through `build-spec`.
3. Keep uncertainty explicit; do not hide low-confidence decisions.

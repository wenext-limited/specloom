---
name: generating-ui-from-ui-spec-ron
description: Use when final ui_spec.ron already exists and an agent must generate target UI code according to explicit user-requested framework, style, and output constraints.
---

# Generating UI from ui_spec.ron

## Overview

Use this skill after `output/specs/ui_spec.ron` is finalized.
Core rule: generate UI for the target the user asked for, not the target the agent prefers.

## When to Use

- `ui_spec.ron` already exists and code generation is next.
- The user specified a target (for example `swiftui`, `react-tailwind`).
- You must keep lookup deterministic and warnings explicit.

Do not use this skill for transform planning (`pre_layout.ron -> transform_plan.json`).

## Required Inputs

- `output/specs/ui_spec.ron` (authoritative generation source)
- `output/agent/agent_context.json`
- `output/agent/search_index.json`
- `output/images/root_<node_id>.png` when visual disambiguation is needed

If agent context files are missing, run:
- `cargo run -p forge -- run-stage build-agent-context`

## Workflow

1. Extract user requirements first:
- target framework
- styling expectations
- output location/format constraints
2. Generate from final `ui_spec.ron` only. Do not fall back to `pre_layout.ron`.
3. Resolve uncertain sections with `forge agent-tool` in order:
- `find-nodes`
- `get-node-info`
- `get-node-screenshot` (only when ambiguous)
4. Implement code section-by-section in the user’s requested target.
5. Keep system-level components simplified (for example iOS status bar), matching the semantic structure already flattened in spec decisions.
6. Emit run artifacts:
- `output/reports/generation_warnings.json`
- `output/reports/generation_trace.json`
7. If confidence is low, continue with best effort and record explicit warnings.

## Quick Reference

| Rule | Requirement |
| --- | --- |
| Source | `ui_spec.ron` is authoritative |
| Target | Must match user request |
| Lookup order | `find-nodes` -> `get-node-info` -> `get-node-screenshot` |
| Uncertainty | Warn, do not silently drop |
| Outputs | UI files + warnings + trace |

## Common Mistakes

- Generating in the wrong framework because it is “faster.”
- Using `pre_layout.ron` instead of final `ui_spec.ron`.
- Skipping `agent-tool` lookup for ambiguous sections.
- Omitting `generation_warnings.json` or `generation_trace.json`.
- Re-expanding system chrome into full decorative detail.

## Red Flags - Stop and Fix

- “I’ll generate SwiftUI even though user asked React.”
- “I can skip trace/warnings to save time.”
- “I already have enough context, no need for node lookup.”

If any red flag appears, pause and return to the workflow above.

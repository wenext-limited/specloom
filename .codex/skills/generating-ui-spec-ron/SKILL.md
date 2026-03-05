---
name: generating-ui-spec-ron
description: Use when converting pre_layout artifacts into final output/specs/ui_spec.ron through screenshot-grounded transform_plan authoring and deterministic build-spec application.
---

# Generating ui_spec.ron

## Overview

Orchestrate this flow only:

`pre_layout.ron + node evidence + screenshot grounding -> transform_plan.json -> build-spec -> ui_spec.ron`

Hard rules:

1. Never hand-edit `ui_spec.ron`.
2. Never copy `pre_layout.ron` into `ui_spec.ron`.

## Required Sub-Skills

1. **REQUIRED SUB-SKILL:** `node-grounding-for-transform`
2. **REQUIRED SUB-SKILL:** `authoring-transform-plan`
3. **CONDITIONAL SUB-SKILL:** `simplifying-system-components`
4. **CONDITIONAL SUB-SKILL:** `inferring-repeat-element-ids`

## Quick Runbook

1. Confirm inputs: `output/specs/pre_layout.ron`, `output/specs/node_map.json`, and root screenshot (`output/images/root_<node_id>.png`) when available.
2. Ground node decisions with `node-grounding-for-transform` (including screenshot checks for root and ambiguous/text-less nodes).
3. Apply optional sub-skills only when evidence requires.
4. Write `output/specs/transform_plan.json` using `authoring-transform-plan`.
5. Run exactly one:
   - `cargo run -p forge -- run-stage build-spec`
   - `forge run-stage build-spec`
6. Confirm final `output/specs/ui_spec.ron` was regenerated.

## Definition of Done

1. `ui_spec.ron` reflects transform decisions.
2. Screenshot grounding was applied for root and ambiguity hotspots (or absence was explicitly noted for fixture/snapshot runs).
3. No manual spec patching occurred.

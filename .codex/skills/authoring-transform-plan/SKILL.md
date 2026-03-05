---
name: authoring-transform-plan
description: Use when writing or updating output/specs/transform_plan.json with valid child_policy decisions and deterministic references.
---

# Authoring transform_plan.json

## Overview

Author a plan that can be applied mechanically without recovery logic.

## Required Shape

```json
{
  "version": "transform_plan/1.0",
  "decisions": []
}
```

Decision fields:

1. `node_id`
2. `suggested_type`
3. `child_policy` (`keep`, `drop`, `remove_self`, `replace_with`)
4. `confidence`
5. `reason`
6. optional `repeat_element_ids`

## Child Policy Ladder

1. `keep`: container semantics remain useful.
2. `drop`: children are decorative/noise.
3. `remove_self`: wrapper should disappear.
4. `replace_with`: keep selected direct children in stable order.

## Validation Checklist

1. No duplicate decision node IDs.
2. Decision node exists in pre-layout.
3. Non-`replace_with` modes have empty `children`.
4. `replace_with` has non-empty direct-child IDs.
5. Root is never `remove_self`.
6. `replace_with` child cannot also be marked `remove_self`.
7. `repeat_element_ids` (if present) are unique and ordered.

## Red Flags

1. "I'll fix invalid edges later."
2. "I'll reorder children manually per run."

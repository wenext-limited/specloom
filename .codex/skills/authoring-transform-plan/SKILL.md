---
name: authoring-transform-plan
description: Use when writing or updating output/specs/transform_plan.json with exact field/type requirements, valid child_policy decisions, and deterministic references.
---

# Authoring transform_plan.json

## Overview

Author a plan that can be applied mechanically without recovery logic.

## Data Definition (Source of Truth)

```ts
type TransformPlan = {
  version: "transform_plan/1.0";
  decisions: TransformDecision[];
};

type TransformDecision = {
  node_id: string;
  suggested_type: SuggestedNodeType;
  child_policy: ChildPolicy;
  repeat_element_ids?: string[];
  confidence: number;
  reason: string;
};

type SuggestedNodeType =
  | "Container"
  | "Instance"
  | "Text"
  | "Image"
  | "Shape"
  | "Vector"
  | "Button"
  | "ScrollView"
  | "HStack"
  | "VStack"
  | "ZStack";

type ChildPolicy = {
  mode: ChildPolicyMode;
  children?: string[];
};

type ChildPolicyMode = "keep" | "drop" | "remove_self" | "replace_with";
```

Notes:

1. Enum values are case-sensitive.
2. Unknown fields are not allowed by contract deserialization.

## Required JSON Shape

```json
{
  "version": "transform_plan/1.0",
  "decisions": []
}
```

Decision required fields:

1. `node_id`
2. `suggested_type`
3. `child_policy`
4. `confidence`
5. `reason`

Decision optional fields:

1. `repeat_element_ids`

## Child Policy Rules

1. `keep`: preserve transformed children.
2. `drop`: remove all children.
3. `remove_self`: remove current node from parent.
4. `replace_with`: keep selected direct children in stable order.

`children` field behavior:

1. Required and non-empty for `replace_with`.
2. Omit (or empty) for `keep`, `drop`, `remove_self`.

## Validation Checklist

1. `version` is exactly `transform_plan/1.0`.
2. No duplicate decision node IDs.
3. Decision node exists in pre-layout.
4. Non-`replace_with` modes do not carry replacement children.
5. `replace_with` contains non-empty direct-child IDs only.
6. Root is never `remove_self`.
7. `replace_with` child cannot also be marked `remove_self`.
8. `repeat_element_ids` (if present) are unique and stably ordered.

## Canonical Valid Example

```json
{
  "version": "transform_plan/1.0",
  "decisions": [
    {
      "node_id": "1:10",
      "suggested_type": "HStack",
      "child_policy": {
        "mode": "replace_with",
        "children": ["1:11", "1:14"]
      },
      "repeat_element_ids": ["row-1", "row-2"],
      "confidence": 0.84,
      "reason": "Keep semantic children in stable visual order"
    }
  ]
}
```

## Red Flags

1. "I'll fix invalid edges later."
2. "I'll reorder children manually per run."

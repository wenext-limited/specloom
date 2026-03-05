---
name: generating-ui-spec-ron
description: Use when an agent must produce final ui_spec.ron from pre_layout.ron, node_map.json, and screenshot grounding with valid transform_plan child-policy decisions.
---

# Generating ui_spec.ron from pre-layout artifacts

## Overview

Use this skill to produce `output/specs/ui_spec.ron` through the required contract:
`pre_layout.ron + node_map.json + screenshot -> transform_plan.json -> build-spec -> ui_spec.ron`.

Core rule: never hand-edit `ui_spec.ron` and never copy `pre_layout.ron` into `ui_spec.ron`.
Final spec must come from transform-plan application.

## When to Use

- You are in `build-spec` transform planning.
- You need semantic node upgrades (`Button`, `ScrollView`, `HStack`, etc.).
- You must decide `child_policy` (`keep`, `drop`, `replace_with`) safely.
- Screenshot evidence is needed to disambiguate weak or text-less structures.

Do not use this skill when you are only inspecting an existing spec without changing transforms.

## Required Inputs

- `output/specs/pre_layout.ron`: authoritative tree structure and parent/child relationships.
- `output/specs/node_map.json`: authoritative per-node properties.
- `output/images/root_<node_id>.png` (or tool-fetched node screenshots): visual grounding only.

Policy: when screenshot and node data conflict, prefer node data and emit a warning.

## repeat_element_ids Policy (Required)

`ui_spec.ron` container-like nodes may include an optional `repeat_element_ids` field.

Current project policy:

1. The agent should infer `repeat_element_ids` during transform planning when repeated-element structure is clear.
2. Only use direct child IDs of the node; keep IDs unique and in stable child order.
3. Infer `repeat_element_ids` only for container-like nodes (`Container`, `Button`).
4. If confidence is low or evidence is weak, leave `repeat_element_ids` absent/empty and record uncertainty in reasoning/warnings.
5. Keep using `transform_plan.json` (`suggested_type` + `child_policy`) as the primary semantic contract; `repeat_element_ids` is complementary metadata.

## Large node_map Handling (Required)

When `node_map.json` is too large to read safely in-context, use `forge agent-tool` instead of loading the full file.

Required lookup order:

1. `forge agent-tool find-nodes --query "<intent text>" --output json`
2. `forge agent-tool get-node-info --node-id <NODE_ID> --output json`
3. `forge agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID> --output json` (only for ambiguity)

This preserves node-data authority while keeping token usage bounded.

## System-Level Component Simplification (Required)

If a node is a common OS/system component (for example iOS status bar), flatten it into a simple semantic structure.

Rules:

1. Do not preserve full decorative/vector internals of system chrome.
2. Prefer a simple container shape (`HStack` or `Container`) with only essential children.
3. Use `child_policy = replace_with` to keep only minimal semantic children in stable order.
4. If children are purely decorative/noise, use `child_policy = drop`.
5. State the simplification explicitly in `reason` (for example: `flattened system component: iOS status bar`).

## Workflow

1. Read `pre_layout.ron` and index node IDs and direct-child sets per parent.
2. Resolve target nodes with `forge agent-tool find-nodes`.
3. Pull authoritative node details with `forge agent-tool get-node-info` (backed by `node_map.json`).
4. Detect system-level components and flatten them per the simplification rules above.
5. Use screenshot only for ambiguity resolution, not as primary truth.
6. Write `output/specs/transform_plan.json` with:
- `version = "transform_plan/1.0"`
- one decision per transformed node
7. Validate before apply:
- no duplicate `decision.node_id`
- all decision nodes exist in pre-layout
- `keep` and `drop` have no `children`
- `replace_with` has non-empty `children`
- each replacement child exists and is a direct child of the decision node
- inferred `repeat_element_ids` (if present) are unique direct child IDs in stable order
- no unknown fields (plan structs use `deny_unknown_fields`)
8. Apply mechanically with exactly one of:
- `cargo run -p forge -- run-stage build-spec`
- `forge run-stage build-spec`
9. Confirm final output is regenerated at `output/specs/ui_spec.ron`.

## Transform Plan Quick Reference

| Field                              | Rule                                                                                                              |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `suggested_type`                   | `Container`, `Instance`, `Text`, `Image`, `Shape`, `Vector`, `Button`, `ScrollView`, `HStack`, `VStack`, `ZStack` |
| `child_policy.mode = keep`         | keep transformed children; do not include `children`                                                              |
| `child_policy.mode = drop`         | remove children; do not include `children`                                                                        |
| `child_policy.mode = replace_with` | include ordered `children` list of direct child IDs                                                               |

## Minimal Example

```json
{
  "version": "transform_plan/1.0",
  "decisions": [
    {
      "node_id": "1:10",
      "suggested_type": "Button",
      "child_policy": { "mode": "drop" },
      "confidence": 0.82,
      "reason": "Container is action-like with label/icon composition"
    }
  ]
}
```

## Common Mistakes

- Copying `pre_layout.ron` to `ui_spec.ron` as a shortcut.
- Ignoring `node_map.json` and inferring from screenshot only.
- Reading full `node_map.json` when targeted `agent-tool` lookup would be safer.
- Modeling system-level components (for example iOS status bar) at full vector/detail depth.
- Using `replace_with` child IDs that are not direct children.
- Hand-editing `ui_spec.ron` instead of applying `transform_plan.json`.
- Silencing conflicts instead of reporting warnings.

## Red Flags - Stop and Fix

- "Fastest path is `cp pre_layout.ron ui_spec.ron`."
- "Screenshot looks right, so I can ignore node_map."
- "I can patch `ui_spec.ron` directly and skip build-spec."
- "I can call a non-existent shortcut like `cargo run -p forge -- build-spec`."

All of these mean the transform contract is being violated.

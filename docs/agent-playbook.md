# Agent Playbook (Single-Agent v1)

This playbook defines how generation agents should use repository tooling in v1.

## Runtime Model

1. Use stateless run-and-consume CLI commands.
2. Do not require a persistent daemon/session server.
3. Read/write deterministic artifacts under `output/`.

## Build-Spec Artifact Order

`build-spec` is a two-step flow (preprocess -> agent transform):

1. `output/specs/pre_layout.ron` (deterministic pre-transform tree)
2. `output/specs/node_map.json` (raw normalized node payload by ID)
3. `output/specs/transform_plan.json` (agent decisions)
4. `output/specs/ui_spec.ron` (final transformed spec)

`build-agent-context` must always run from the final transformed `ui_spec.ron`, not from pre-layout.

## Transform Plan Contract

`transform_plan.json` decisions are authoritative for high-level node typing and child handling:

1. `suggested_type` defines the target node kind (`Button`, `ScrollView`, `HStack`, etc.).
2. `child_policy.mode=keep` preserves transformed children.
3. `child_policy.mode=drop` removes all children.
4. `child_policy.mode=replace_with` keeps only listed child IDs (ordered list).

Do not apply additional deterministic semantic collapse rules after transform-plan application.

## Required Tool Order

1. `find_nodes` first for lookup from UI context text/structure.
2. `get_node_info` for selected node IDs.
3. `get_node_screenshot` when lookup is ambiguous or text-less elements need visual confirmation.
4. `get_asset` for icon/image extraction when applicable.

## Mismatch Policy

1. On lookup mismatch, continue generation with best effort.
2. Never silently drop uncertainty.
3. Emit structured warnings to `output/reports/generation_warnings.json`.

Expected warning types:

1. `NODE_NOT_FOUND`
2. `LOW_CONFIDENCE_MATCH`
3. `MULTIPLE_CANDIDATES`
4. `SCREENSHOT_NODE_MISMATCH`
5. `UNSUPPORTED_STYLE_MAPPING`

## Trace Policy

Record tool execution trace in `output/reports/generation_trace.json`:

1. tool name
2. status
3. query/input
4. selected candidate node IDs

## Output Expectations

For every generation run:

1. produce target UI output files
2. produce warning artifact
3. produce trace artifact

Do not claim successful completion without all three output classes.

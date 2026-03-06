# Agent Playbook (Single-Agent v1)

This playbook defines how generation agents should use repository tooling in v1.

## Runtime Model

1. Use stateless run-and-consume CLI commands.
2. Do not require a persistent daemon/session server.
3. Read/write deterministic artifacts under `output/`.

## Operator Run Order (End-to-End)

1. Run deterministic pipeline (`specloom generate --input fixture` or live `specloom generate --input live --figma-url "<FIGMA_URL>"`).
2. Build bundle: `specloom prepare-llm-bundle --figma-url "<FIGMA_URL>" --target <TARGET> --intent "<INTENT>"`.
3. Generate UI: `specloom generate-ui --bundle output/agent/llm_bundle.json`.

`prepare-llm-bundle` is responsible for transform readiness. It must not package a stale or empty semantic transform state; when `transform_plan.json` is missing or empty, it authors a non-empty plan, then refreshes `build-spec` and `build-agent-context` before writing the bundle.

Expected outputs:

1. `output/agent/llm_bundle.json`
2. generated target code under `output/generated/<target>/...`
3. `output/reports/generation_warnings.json`
4. `output/reports/generation_trace.json`

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
4. `child_policy.mode=remove_self` removes the current node from its parent.
5. `child_policy.mode=replace_with` keeps only listed child IDs (ordered list).
6. `repeat_element_ids` (optional on each decision) overrides repeat metadata for that node in final `ui_spec.ron`.

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

1. produce target UI output files under `output/generated/<target>/...`
2. produce warning artifact at `output/reports/generation_warnings.json`
3. produce trace artifact at `output/reports/generation_trace.json`

Do not claim successful completion without all three output classes.

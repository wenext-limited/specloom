# Agent Playbook (Single-Agent v1)

This playbook defines how generation agents should use repository tooling in v1.

## Runtime Model

1. Use stateless run-and-consume CLI commands.
2. Do not require a persistent daemon/session server.
3. Read/write deterministic artifacts under `output/`.

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

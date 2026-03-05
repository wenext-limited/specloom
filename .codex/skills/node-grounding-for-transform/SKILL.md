---
name: node-grounding-for-transform
description: Use when transform decisions require authoritative node evidence with screenshot grounding from root and ambiguous nodes.
---

# Node Grounding for Transform

## Hard Rule for node_map.json

1. Do not read `output/specs/node_map.json` directly (`cat`, `sed`, `jq`, full-file loads).
2. Only check existence: `test -f output/specs/node_map.json`.
3. Read node data via `forge agent-tool` commands only.

## Evidence Priority

1. Node contracts from `agent-tool get-node-info` are authoritative.
2. Screenshot grounding is required for visual confirmation (root and ambiguity hotspots).
3. If screenshot and node data conflict, prefer node data and state uncertainty.

## Required Lookup Order

1. `forge agent-tool find-nodes --query "<intent>" --output json`
2. `forge agent-tool get-node-info --node-id <NODE_ID> --output json`
3. `forge agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID> --output json` for root and ambiguous/text-less targets

## Size/Mode Rules

1. `node_map.json` may be very large; never load it directly in context.
2. `live`: screenshot grounding is required for root and ambiguity hotspots.
3. `fixture`/`snapshot`: use screenshot artifacts if present; if absent, proceed with node evidence and explicitly note screenshot unavailability.
4. Never block transform planning on screenshot absence alone.

## Done Criteria

For each transformed node, capture:

1. node id + concrete node evidence
2. intended target type
3. reason grounded in fields
4. screenshot-backed note for root/ambiguous nodes (or explicit absence note)

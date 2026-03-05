---
name: node-grounding-for-transform
description: Use when transform decisions require authoritative node evidence from node_map or targeted agent-tool lookup.
---

# Node Grounding for Transform

## Evidence Priority

1. Node contracts (`node_map.json` / `get-node-info`) are authoritative.
2. Screenshot is only for ambiguity resolution.
3. If they conflict, prefer node data and state uncertainty.

## Required Lookup Order

1. `forge agent-tool find-nodes --query "<intent>" --output json`
2. `forge agent-tool get-node-info --node-id <NODE_ID> --output json`
3. `forge agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID> --output json` (ambiguity only)

## Size/Mode Rules

1. Large `node_map.json`: use targeted tool lookups, not full-file loading.
2. `live`: use screenshot lookup when node evidence is still ambiguous.
3. `fixture`/`snapshot`: continue with node evidence if screenshot is unavailable.
4. Never block transform planning on screenshot absence alone.

## Done Criteria

For each transformed node, capture:

1. node id + concrete evidence
2. intended target type
3. reason grounded in fields (not intuition)

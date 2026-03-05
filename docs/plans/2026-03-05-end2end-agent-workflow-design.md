# End-to-End Agent Workflow Design (Prepare + Generate)

## Context

The current Specloom mainline is deterministic through:

1. `fetch`
2. `normalize`
3. `build-spec`
4. `build-agent-context`
5. `export-assets`

Agent lookup tooling exists (`find-nodes`, `get-node-info`, `get-node-screenshot`), but end-to-end code generation still requires manual orchestration across artifacts and prompts.

The product direction for the next 1-2 weeks is feature velocity: a developer provides a Figma URL and generation intent, and Specloom handles the rest with minimal manual steps.

## Approved Decisions

Validated decisions for this design:

1. Prioritize feature velocity over broader hardening or docs-only work.
2. Target a 1-2 week implementation horizon.
3. Build an end-to-end agent workflow.
4. Use a two-command model:
   - `prepare-llm-bundle`
   - `generate-ui`
5. Use project skills and agent docs as explicit instruction inputs to the LLM.

## Approaches Considered

### 1) Single-command vertical slice (`generate-ui` only)

Pros:
1. Best developer UX.
2. Closest to "do the rest" immediately.

Cons:
1. Harder to debug and replay.
2. Larger integration risk for v1.

### 2) Two-command flow (`prepare-llm-bundle` then `generate-ui`) **Chosen**

Pros:
1. Fast to ship with clear boundaries.
2. Deterministic checkpoint artifact for replay and debugging.
3. Preserves current stage contracts without large refactor.

Cons:
1. Slightly less streamlined than a one-command UX.

### 3) Thin wrapper around existing manual commands

Pros:
1. Fastest initial implementation.

Cons:
1. Technical debt in orchestration logic.
2. Weak long-term contract and auditability.

## Architecture

### 1) `prepare-llm-bundle` command

Responsibilities:

1. Accept `--figma-url` (or equivalent live inputs), `--target`, and user `--intent`.
2. Execute or verify deterministic prerequisite stages:
   - `fetch -> normalize -> build-spec -> build-agent-context -> export-assets`
3. Load required guidance sources:
   - `.codex/SKILLS.md`
   - selected skill docs under `.codex/skills/`
   - `docs/agent-playbook.md`
   - `docs/figma-ui-coder.md`
4. Emit one bundle artifact:
   - `output/agent/llm_bundle.json`

### 2) `generate-ui` command

Responsibilities:

1. Accept `--bundle output/agent/llm_bundle.json`.
2. Invoke the agent/LLM runner using only bundle-defined inputs.
3. Write generated code to:
   - `output/generated/<target>/...`
4. Always produce report artifacts:
   - `output/reports/generation_warnings.json`
   - `output/reports/generation_trace.json`

### 3) Agent runner boundary

Introduce a `core` abstraction (for example a trait/module boundary) so:

1. CLI and orchestration depend on a stable generation interface.
2. LLM backend/provider can be swapped without changing bundle contract.
3. Tests can run with a deterministic mock runner.

## `llm_bundle.json` Contract (v1)

Proposed shape:

1. `version` (for example `llm_bundle/1.0`)
2. `request`
   - target framework/runtime
   - user intent/instructions
3. `figma`
   - source URL
   - resolved `file_key`
   - resolved `root_node_id`
4. `artifacts`
   - path + hash for `ui_spec.ron`
   - path + hash for `agent_context.json`
   - path + hash for `search_index.json`
   - path + hash for root screenshot (if present)
   - path + hash for `asset_manifest.json`
5. `instructions`
   - embedded `docs/agent-playbook.md`
   - embedded `docs/figma-ui-coder.md`
   - selected skill entries and source paths
6. `tool_contract`
   - available tool names and invocation notes

Contract principles:

1. Deterministic ordering and serialization.
2. Explicit references only; no implicit filesystem discovery in `generate-ui`.
3. Fail early on missing required inputs during `prepare-llm-bundle`.

## Data Flow

1. Developer runs `prepare-llm-bundle` with Figma URL, target, and intent.
2. Deterministic stages run and produce canonical artifacts.
3. Bundle captures artifact references/hashes and instruction payloads.
4. Developer runs `generate-ui --bundle ...`.
5. Agent executes with bundle context and tool-assisted lookup.
6. Generated code and report artifacts are written.

## Error Handling

### Prepare-time errors (hard fail)

1. Missing required deterministic artifacts.
2. Invalid Figma URL or unresolved live fetch requirements.
3. Missing referenced skill/doc inputs required by bundle policy.
4. Invalid bundle schema serialization.

### Generate-time behavior

1. Tool mismatch/ambiguity remains non-fatal and is recorded as warnings.
2. Provider/runner failure fails `generate-ui` explicitly.
3. Partial generation keeps trace output for diagnosis.

## Testing and Verification

### Unit tests

1. `llm_bundle.json` round-trip and schema validation.
2. Deterministic bundle ordering and stable hash output.
3. Skill/doc selection and embedding rules.

### Integration tests

1. Fixture path:
   - `prepare-llm-bundle` emits valid bundle and expected artifact references.
2. Mock runner path:
   - `generate-ui --bundle ...` writes outputs + warnings + trace.
3. Replay path:
   - same bundle input with mock runner yields byte-stable outputs.
4. Failure path:
   - missing skill/doc or artifact fails with actionable message.

### Verification gates

1. `cargo check --workspace`
2. `cargo test --workspace`
3. `bash scripts/verify_workspace.sh`

## Delivery Scope (1-2 Weeks)

1. Add `prepare-llm-bundle` CLI + core wiring.
2. Add `llm_bundle.json` contract + tests.
3. Add `generate-ui` CLI + mockable runner interface.
4. Wire skills/playbook/doc ingestion into bundle.
5. Add fixture + mock integration coverage.
6. Update CLI and root docs with new operator flow.

## Risks and Mitigations

1. Risk: instruction payload bloat reduces generation quality.
   Mitigation: include only selected skills relevant to request target and phase.
2. Risk: nondeterministic generation output complicates tests.
   Mitigation: test determinism on bundle/build side and use deterministic mock runner for codegen tests.
3. Risk: drift between declared tool contract and CLI surface.
   Mitigation: add contract consistency tests between bundle tool list and `agent-tool` commands.

## Outcome

This design delivers a practical end-to-end agent workflow quickly while preserving Specloom's deterministic core and artifact-first auditability. It enables "give Figma URL + intent, then generate" with a stable intermediate checkpoint that supports replay, debugging, and incremental future automation into a single-command UX.

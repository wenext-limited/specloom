# Proposal: Forge Deterministic Figma-to-Spec + Agent-Context Pipeline (Rust 2024)

This proposal is the canonical project baseline as of **March 5, 2026**.
It reflects what is implemented on `main` today and the immediate roadmap from the latest completed boards.

## 1. Why This Project Exists

Modern design-to-code flows fail in two predictable ways:

1. They pass too much raw design data into generation, which hurts reliability and traceability.
2. They hide uncertainty, which makes failures hard to debug and hard to trust.

Forge addresses this by splitting the flow into deterministic artifact stages, then exposing a small, explicit agent tooling surface.

## 2. Goals

1. Convert Figma snapshots into stable, reviewable intermediate artifacts.
2. Keep semantic decisions explicit through a versioned transform contract (`transform_plan.json`).
3. Support agent-assisted lookup/generation with deterministic search + explicit warning/trace outputs.
4. Preserve reproducibility across runs for identical inputs.

## 3. Non-Goals (Current Mainline)

1. Direct target UI generation as a default pipeline stage.
2. Mandatory always-on daemon/session runtime.
3. Silent fallback behavior when confidence is low or lookup fails.
4. Reintroducing removed default stages (`infer-layout`, legacy SwiftUI codegen stages) into `generate`.

## 4. Current End-to-End Architecture

```text
input (fixture | live | snapshot)
  -> fetch
  -> normalize
  -> build-spec
  -> build-agent-context
  -> export-assets
```

Default `generate` order:

1. `fetch`
2. `normalize`
3. `build-spec`
4. `build-agent-context`
5. `export-assets`

## 5. Stage Contracts and Artifacts

### `fetch`

Input modes:

1. `fixture` (default for `fetch`)
2. `live` (`--file-key` + `--node-id` or `--figma-url`, plus token)
3. `snapshot` (`--snapshot-path`)

Output:

1. `output/raw/fetch_snapshot.json`

### `normalize`

Input:

1. `output/raw/fetch_snapshot.json`

Output:

1. `output/normalized/normalized_document.json`

### `build-spec`

Inputs:

1. `output/normalized/normalized_document.json`
2. Optional seeded `output/specs/transform_plan.json`

Outputs (in this order):

1. `output/specs/pre_layout.ron`
2. `output/specs/node_map.json`
3. `output/specs/transform_plan.json`
4. `output/specs/ui_spec.ron`

Behavior:

1. `pre_layout.ron` is deterministically built from normalized nodes.
2. `node_map.json` is emitted with stable key ordering (`BTreeMap`).
3. If `transform_plan.json` exists, it is loaded + validated.
4. If missing, an empty default transform plan is created.
5. Final `ui_spec.ron` is produced only by applying the transform plan.

Important current boundary:

1. `build-spec` does **not** directly invoke an LLM in mainline code.
2. Agent-driven plan authoring is currently an external/operator step that writes `transform_plan.json`.

### `build-agent-context`

Input:

1. Final transformed `output/specs/ui_spec.ron`

Outputs:

1. `output/agent/agent_context.json`
2. `output/agent/search_index.json`
3. `output/images/root_<node_id>.png` (live mode, cached if already present)

### `export-assets`

Input:

1. `output/normalized/normalized_document.json`

Output:

1. `output/assets/asset_manifest.json`

## 6. Core Data Contracts (Current Versions)

1. Raw snapshot: `snapshot_version = "1.0"`
2. Normalized document: `schema_version = "1.0"`
3. Layout inference record (library): `decision_version = "1.0"`
4. UI spec: `UI_SPEC_VERSION = "2.0"`
5. Transform plan: `version = "transform_plan/1.0"`
6. Node map: `version = "node_map/1.0"`
7. Agent context: `version = "agent_context/1.0"`
8. Search index: `version = "search_index/1.0"`
9. Asset manifest: `manifest_version = "1.0"`
10. Generation warnings: `version = "generation_warnings/1.0"`
11. Generation trace: `version = "generation_trace/1.0"`

## 7. Transform Plan Policy

`transform_plan.json` is the semantic control surface for final `ui_spec.ron`.

Each decision includes:

1. `node_id`
2. `suggested_type` (`Container`, `Button`, `HStack`, etc.)
3. `child_policy` (`keep`, `drop`, `remove_self`, `replace_with`)
4. `confidence`
5. `reason`
6. Optional `repeat_element_ids`

Validation enforces:

1. Supported version.
2. No duplicate decisions per node.
3. Decision nodes and replacement children must exist.
4. `replace_with` requires non-empty direct-child list.
5. `remove_self` cannot target root.
6. `replace_with` cannot reference a child that is also removed.
7. `repeat_element_ids` must be unique when provided.

## 8. Agent Runtime Model (Current)

Run-and-consume (stateless) is the default model.

Implemented tool commands:

1. `agent-tool find-nodes`
2. `agent-tool get-node-info`
3. `agent-tool get-node-screenshot`

Lookup/report behavior:

1. Deterministic fuzzy ranking with stable tie-break (`score desc`, `node_id asc`).
2. Low-confidence/ambiguous/no-match cases append warnings to `output/reports/generation_warnings.json`.
3. Tool usage appends events to `output/reports/generation_trace.json`.

Current known gap:

1. `get_asset` appears in context metadata but is not yet exposed as a CLI agent tool command.

## 9. Crate Responsibilities (Current Workspace)

1. `crates/cli` (`forge-figma-pipeline` package, `forge` binary): CLI command surface and output formatting.
2. `crates/orchestrator` (`forge-figma-core` package): core contracts and stage execution runtime including fetch/screenshot client APIs, normalization, ui-spec transform logic, agent-context lookup, and asset manifest generation.

## 10. Determinism and Safety Guarantees

1. Stable stage order and explicit artifact paths.
2. Stable serialization and ordering in key contracts.
3. Explicit actionable errors for unknown stage, missing artifacts, and fetch/config issues.
4. No silent fallback for required upstream artifacts.
5. Live mode requires explicit credentials (`FIGMA_TOKEN` or `--figma-token`).
6. Figma URL parsing validates host and required `node-id`.

## 11. Operator Workflow (Canonical CLI Pattern)

```bash
cargo run -p forge-figma-pipeline -- stages
cargo run -p forge-figma-pipeline -- fetch --input fixture
cargo run -p forge-figma-pipeline -- generate --input fixture
cargo run -p forge-figma-pipeline -- generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE>?node-id=<NODE_ID>"
cargo run -p forge-figma-pipeline -- agent-tool find-nodes --query "login button" --output json
cargo run -p forge-figma-pipeline -- agent-tool get-node-info --node-id <NODE_ID>
```

Defaults:

1. `fetch` defaults to `--input fixture`.
2. `generate` defaults to `--input live`.

## 12. Documentation Drift Note

Several historical plans/boards describe earlier architectures that are no longer the active mainline (for example `infer-layout` in default run order, SwiftUI AST/codegen crates, or `llm_*` stage wiring).

For implementation reality, treat these as canonical first:

1. `docs/proposal.md` (this document)
2. `README.md` (command/operator usage)
3. `docs/agent-playbook.md` (tooling and reporting policy)
4. Phase 13/14 boards under `docs/plans/boards/2026-03-05-*`
5. `.codex/SKILLS.md` and skill docs under `.codex/skills/`

## 13. Next Milestones

1. Add first-class transform-plan authoring flow (agent-assisted) inside the pipeline boundary, not only as an external pre-seeded file.
2. Add `get_asset` command to match tool metadata and close the agent tool surface gap.
3. Add review aggregation stage (or equivalent) over normalization/transform/lookup/asset warnings.
4. Add target code generation stage(s) after transform and reporting contracts are fully stable.
5. Keep all additions versioned, deterministic, and independently executable.

## 14. Verification Gates

For code changes:

```bash
cargo check --workspace
cargo test --workspace
```

For docs-only updates: perform consistency checks against stage names, artifact paths, and command examples.

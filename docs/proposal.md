# Proposal: Figma Node Tree -> Spec + Agent Context Pipeline (Rust 2024)

For current command usage and artifact expectations, see the root [`README.md`](/Users/wendell/Developer/forge/README.md).

## 1. Objective

This document reflects the **currently implemented** repository behavior (as of 2026-03-05), not the earlier aspirational SwiftUI/LLM architecture drafts.

Primary objective:

1. Build a deterministic Rust pipeline that transforms Figma node data into stable intermediate artifacts for downstream UI generation workflows.

Current implemented outcomes:

1. Fetch raw Figma snapshot data (fixture, live API, or snapshot replay).
2. Normalize snapshot JSON into a canonical node document.
3. Build a pre-layout spec and a transformed UI spec.
4. Build agent-facing context/search artifacts for deterministic lookup tooling.
5. Build an asset manifest from normalized image fills.

Out of scope in the current implementation:

1. Direct SwiftUI file generation as part of default pipeline stages.
2. LLM bundle/generation stages (`prepare-llm-bundle`, `generate-ui`, etc.).
3. A wired `review-report` stage in orchestrator default flow.

## 2. Current End-to-End Architecture

```text
Figma fixture/live/snapshot input
   │
   ▼
fetch
  -> output/raw/fetch_snapshot.json
   │
   ▼
normalize
  -> output/normalized/normalized_document.json
   │
   ▼
build-spec
  -> output/specs/pre_layout.ron
  -> output/specs/node_map.json
  -> output/specs/transform_plan.json
  -> output/specs/ui_spec.ron
   │
   ▼
build-agent-context
  -> output/agent/agent_context.json
  -> output/agent/search_index.json
  -> (live mode only) output/images/root_<node_id>.png
   │
   ▼
export-assets
  -> output/assets/asset_manifest.json
```

Default `generate` stage order:

1. `fetch`
2. `normalize`
3. `build-spec`
4. `build-agent-context`
5. `export-assets`

## 3. Workspace Shape (Current)

Current workspace members in [`Cargo.toml`](/Users/wendell/Developer/forge/Cargo.toml):

1. `crates/cli` (package name: `forge`)
2. `crates/figma_client`
3. `crates/figma_normalizer`
4. `crates/layout_infer`
5. `crates/ui_spec`
6. `crates/asset_pipeline`
7. `crates/orchestrator`
8. `crates/agent_context`

Current responsibility split:

1. `figma_client`: fixture + live Figma fetch and screenshot fetch contracts.
2. `figma_normalizer`: canonical normalized document + warning surfacing + passthrough fields.
3. `layout_infer`: deterministic layout inference contracts/heuristics (currently library-only, not stage-wired).
4. `ui_spec`: pre-layout build, transform-plan validation, transformed `UiSpec` output.
5. `agent_context`: context/search models and deterministic ranking logic.
6. `asset_pipeline`: deterministic asset manifest extraction from normalized image fills.
7. `orchestrator`: stage execution, artifact I/O, actionable errors, agent-tool operations.
8. `cli` (`forge`): command parsing and text/json output interfaces.

## 4. Stage Contracts and Artifacts

### 4.1 `fetch`

Inputs:

1. `fixture` mode (default for `fetch`).
2. `live` mode (`--file-key` + `--node-id` or `--figma-url`, plus token via env/flag).
3. `snapshot` mode (`--snapshot-path`).

Output:

1. `output/raw/fetch_snapshot.json` (`snapshot_version = "1.0"`).

### 4.2 `normalize`

Input:

1. `output/raw/fetch_snapshot.json`.

Output:

1. `output/normalized/normalized_document.json`.
2. Includes deterministic node traversal output plus explicit normalization warnings.

### 4.3 `build-spec`

Input:

1. `output/normalized/normalized_document.json`.

Outputs:

1. `output/specs/pre_layout.ron`
2. `output/specs/node_map.json` (`version = "node_map/1.0"`)
3. `output/specs/transform_plan.json` (`version = "transform_plan/1.0"`)
4. `output/specs/ui_spec.ron`

Behavior notes:

1. If `output/specs/transform_plan.json` already exists, it is loaded and validated.
2. Otherwise, default empty transform plan is used.
3. Final `ui_spec.ron` is produced by applying the transform plan to pre-layout spec.

### 4.4 `build-agent-context`

Input:

1. `output/specs/ui_spec.ron`.

Outputs:

1. `output/agent/agent_context.json` (`version = "agent_context/1.0"`).
2. `output/agent/search_index.json` (`version = "search_index/1.0"`).

Additional behavior:

1. In live mode, downloads root-node screenshot into `output/images/root_<node_id>.png` if not already present.

### 4.5 `export-assets`

Input:

1. `output/normalized/normalized_document.json`.

Output:

1. `output/assets/asset_manifest.json` (`manifest_version = "1.0"`).

Behavior notes:

1. Extracts image-fill assets.
2. Emits deterministic ordering and deterministic output filenames.
3. Surfaces missing image refs as warnings in the manifest.

## 5. Agent Tooling Surface (Current)

Implemented stateless tools under `forge agent-tool`:

1. `find-nodes` (deterministic fuzzy ranking over `search_index.json`)
2. `get-node-info` (node details lookup by ID)
3. `get-node-screenshot` (live Figma images API call)

Tool side-effects:

1. Appends warning records to `output/reports/generation_warnings.json` for no-match/low-confidence/ambiguous/not-found flows.
2. Appends trace events to `output/reports/generation_trace.json`.

## 6. CLI Workflow (Current)

Canonical examples (package name is `forge`):

```bash
cargo run -p forge -- stages
cargo run -p forge -- fetch --input fixture
cargo run -p forge -- fetch --input live --file-key <FILE_KEY> --node-id <NODE_ID>
cargo run -p forge -- fetch --input snapshot --snapshot-path <PATH>
cargo run -p forge -- generate --input fixture
cargo run -p forge -- generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE>?node-id=<NODE_ID>"
cargo run -p forge -- agent-tool find-nodes --query "login button" --output json
cargo run -p forge -- agent-tool get-node-info --node-id <NODE_ID>
```

Input defaults:

1. `fetch` defaults to `--input fixture`.
2. `generate` defaults to `--input live`.

Output modes:

1. `text` (default)
2. `json`

## 7. Determinism and Safety Constraints

Current deterministic guarantees implemented in code/tests:

1. Stable stage ordering and explicit artifact paths.
2. Stable serialization for core contracts (JSON and RON snapshots in tests).
3. Deterministic sorting in search ranking tie-breaks and asset manifest entries.
4. Explicit unknown-stage and missing-artifact actionable errors.
5. No silent fallback for required upstream artifacts.

Current safety constraints:

1. Live fetch requires explicit token via `FIGMA_TOKEN` or `--figma-token`.
2. Quick-link parsing validates host (`figma.com`/`www.figma.com`) and `node-id`.
3. Fetch and screenshot API failures are surfaced with actionable messaging.

## 8. Implemented Data Contracts (Version Snapshot)

1. Raw snapshot: `snapshot_version = "1.0"` (`figma_client`)
2. Normalized document: `schema_version = "1.0"` (`figma_normalizer`)
3. Layout inference record: `decision_version = "1.0"` (`layout_infer`, library-only)
4. UI spec: `UI_SPEC_VERSION = "2.0"` (`ui_spec`)
5. Transform plan: `version = "transform_plan/1.0"` (`ui_spec`)
6. Agent context: `version = "agent_context/1.0"` (`agent_context`)
7. Search index: `version = "search_index/1.0"` (`agent_context`)
8. Asset manifest: `manifest_version = "1.0"` (`asset_pipeline`)
9. Generation warnings: `version = "generation_warnings/1.0"` (`agent_context`)
10. Generation trace: `version = "generation_trace/1.0"` (`agent_context`)

## 9. Explicit Gaps vs Long-Term Vision

The repository name/history references SwiftUI generation, but current code is intentionally positioned as a deterministic **spec + agent-context foundation**.

Not yet wired into the pipeline:

1. `infer-layout` stage execution in orchestrator (despite `layout_infer` crate availability).
2. Direct UI codegen stage(s).
3. Aggregated review report stage in default flow.
4. LLM bundle and model execution stages.

## 10. Next Milestones (Proposal-Aligned)

Recommended next phases from the current baseline:

1. Wire `layout_infer` into orchestrator as a first-class stage with persisted artifact output.
2. Thread layout decisions into `build-spec` transform-plan generation.
3. Add explicit review-report stage that aggregates normalization/inference/asset/agent warnings.
4. Introduce target codegen stage(s) after contract and review surfaces are stable.
5. Keep all new stages deterministic, versioned, and independently executable.

## 11. Verification Gates

For code changes:

```bash
cargo check --workspace
cargo test --workspace
```

For docs-only changes (like this update), perform a self-check that:

1. stage names and artifacts match orchestrator/CLI code,
2. crate layout matches workspace manifest,
3. command examples match current package/binary behavior.

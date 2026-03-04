# Forge

Figma node tree to spec-first UI pipeline workspace in Rust 2024.

This repository implements a deterministic, stage-based pipeline that produces:

1. normalized/intermediate JSON artifacts
2. spec artifacts (`pre_layout.ron`, `node_map.json`, `transform_plan.json`, `ui_spec.ron`)
3. agent context/search artifacts for lookup tooling
4. asset manifest metadata

## Quickstart

Run from the repository root:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p cli -- fetch --input fixture
cargo run -p cli -- generate --input fixture
```

The `generate` command runs the full pipeline in order and writes artifacts under `output/`.

## Live Figma Quickstart

Set your token once (or pass `--figma-token` per command):

```bash
export FIGMA_TOKEN="<YOUR_FIGMA_PERSONAL_ACCESS_TOKEN>"
```

Fetch and inspect a real node snapshot:

```bash
cargo run -p cli -- fetch --input live --file-key <FILE_KEY> --node-id <NODE_ID>
cargo run -p cli -- fetch --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
```

Run the full pipeline from live Figma data:

```bash
cargo run -p cli -- generate --input live --file-key <FILE_KEY> --node-id <NODE_ID>
cargo run -p cli -- generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
```

## Inputs and Outputs

Input modes:

1. `fixture`: uses a built-in deterministic payload for local/testing runs.
2. `live`: calls the Figma API with either `--file-key` + `--node-id`, or a single `--figma-url`.
3. `snapshot`: loads an existing raw snapshot via `--snapshot-path` and reuses it as fetch output.
4. Defaults:
5. `fetch` defaults to `--input fixture`.
6. `generate` defaults to `--input live`.
7. `FIGMA_TOKEN` env (or `--figma-token`) is required for `live`.
8. Downstream stages read prior artifacts from `output/`.

Generated artifacts:

| Stage | Output Directory | Artifact |
| --- | --- | --- |
| `fetch` | `output/raw` | `output/raw/fetch_snapshot.json` |
| `normalize` | `output/normalized` | `output/normalized/normalized_document.json` |
| `build-spec` | `output/specs` | `output/specs/pre_layout.ron`, `output/specs/node_map.json`, `output/specs/transform_plan.json`, `output/specs/ui_spec.ron` |
| `build-agent-context` | `output/agent` | `output/agent/agent_context.json`, `output/agent/search_index.json` |
| `export-assets` | `output/assets` | `output/assets/asset_manifest.json` |

Within `build-spec`, artifacts are produced in this order:

1. `pre_layout.ron` (deterministic pre-transform tree)
2. `node_map.json` (raw normalized node payload map)
3. `transform_plan.json` (agent decisions with `child_policy`)
4. `ui_spec.ron` (final transformed spec consumed downstream)

## CLI Commands

List stage output directories:

```bash
cargo run -p cli -- stages
cargo run -p cli -- stages --output json
```

Run one stage:

```bash
cargo run -p cli -- run-stage fetch
cargo run -p cli -- run-stage normalize --output json
```

Run fetch stage directly (fixture or live):

```bash
cargo run -p cli -- fetch
cargo run -p cli -- fetch --input live --file-key <FILE_KEY> --node-id <NODE_ID>
cargo run -p cli -- fetch --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
cargo run -p cli -- fetch --input snapshot --snapshot-path <PATH_TO_FETCH_SNAPSHOT_JSON>
```

Run full pipeline:

```bash
cargo run -p cli -- generate --input fixture
cargo run -p cli -- generate --input fixture --output json
cargo run -p cli -- generate --input live --file-key <FILE_KEY> --node-id <NODE_ID>
cargo run -p cli -- generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
cargo run -p cli -- generate --input snapshot --snapshot-path <PATH_TO_FETCH_SNAPSHOT_JSON>
```

Run agent lookup tools (stateless run-and-consume):

```bash
cargo run -p cli -- agent-tool find-nodes --query "welcome back" --output json
cargo run -p cli -- agent-tool get-node-info --node-id <NODE_ID>
cargo run -p cli -- agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID>
```

## CLI Workflow Matrix

| Goal | Command | Output Mode |
| --- | --- | --- |
| Inspect all stage output directories | `cargo run -p cli -- stages` | text (default) |
| Inspect all stage output directories as machine-readable data | `cargo run -p cli -- stages --output json` | json |
| Run fetch stage with fixture input | `cargo run -p cli -- fetch --input fixture` | text (default) |
| Run fetch stage with live Figma input | `cargo run -p cli -- fetch --input live --file-key <file> --node-id <node>` | text (default) |
| Run fetch stage with Figma quick link input | `cargo run -p cli -- fetch --input live --figma-url "<figma-url>"` | text (default) |
| Run fetch stage with existing snapshot artifact | `cargo run -p cli -- fetch --input snapshot --snapshot-path <path>` | text (default) |
| Run one stage with human-readable output | `cargo run -p cli -- run-stage <stage>` | text (default) |
| Run one stage with machine-readable output | `cargo run -p cli -- run-stage <stage> --output json` | json |
| Run end-to-end pipeline with fixture input and per-stage artifact lines | `cargo run -p cli -- generate --input fixture` | text (default) |
| Run end-to-end pipeline with live Figma input | `cargo run -p cli -- generate --input live --file-key <file> --node-id <node>` | text (default) |
| Run end-to-end pipeline with Figma quick link input | `cargo run -p cli -- generate --input live --figma-url "<figma-url>"` | text (default) |
| Run end-to-end pipeline from existing snapshot artifact | `cargo run -p cli -- generate --input snapshot --snapshot-path <path>` | text (default) |
| Run end-to-end pipeline with fixture input and structured stage results | `cargo run -p cli -- generate --input fixture --output json` | json |
| Find candidate nodes via deterministic fuzzy lookup | `cargo run -p cli -- agent-tool find-nodes --query "<text>" --output json` | text/json |
| Read indexed node details | `cargo run -p cli -- agent-tool get-node-info --node-id <id>` | text/json |
| Fetch node screenshot directly from Figma images API | `cargo run -p cli -- agent-tool get-node-screenshot --file-key <file> --node-id <node>` | text/json |

Notes:

1. Valid stages are: `fetch`, `normalize`, `build-spec`, `build-agent-context`, and `export-assets`.
2. Invalid stage execution returns exit code `2` with an explicit error message.
3. `generate` runs deterministic default stages sequentially: `fetch`, `normalize`, `build-spec`, `build-agent-context`, and `export-assets`.
4. `generate` defaults to `--input live`; pass `--input fixture` for deterministic local runs.
5. Agent tool commands are stateless run-and-consume invocations; no background daemon is required.

## Scope

In-scope right now:

1. deterministic stage orchestration and artifact handoff
2. fixture, live, and snapshot fetch input modes for `fetch` and `generate`
3. fixture-backed end-to-end generate coverage

Not yet in scope:

1. full fidelity translation of advanced visual effects/interactions
2. production app architecture/accessibility/localization generation

## Verification

Use these gates before claiming completion:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/verify_workspace.sh
```

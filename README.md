# Specloom

Figma node tree to spec-first UI pipeline workspace.

This repository implements a deterministic, stage-based pipeline that produces:

1. normalized/intermediate JSON artifacts
2. spec artifacts (`pre_layout.ron`, `node_map.json`, `transform_plan.json`, `ui_spec.ron`)
3. agent context/search artifacts for lookup tooling
4. asset manifest metadata
5. LLM bundle + generated target UI outputs with warning/trace reports

## Workspace Crates

1. `crates/core` (`specloom-core`): core contracts and stage execution runtime.
2. `crates/cli` (`specloom-cli`): command-line interface for running stages and lookup tools.

CLI usage docs live in [`crates/cli/README.md`](crates/cli/README.md).

## Pipeline Artifacts

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

For live `generate` runs, `build-agent-context` also downloads the root-node screenshot to:

1. `output/images/root_<node_id_with_colon_replaced_by_underscore>.png`

## End-to-End Agent Workflow

```bash
# 1) Run deterministic pipeline (fixture or live)
specloom generate --input fixture
specloom generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"

# 2) Build agent bundle for generation
specloom prepare-llm-bundle --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>" --target react-tailwind --intent "Generate login screen code"

# 3) Generate target UI from the prepared bundle
specloom generate-ui --bundle output/agent/llm_bundle.json
```

Expected outputs for this flow:

1. `output/agent/llm_bundle.json`
2. generated target code under `output/generated/<target>/...`
3. `output/reports/generation_warnings.json`
4. `output/reports/generation_trace.json`

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

## Open-Source Project Docs

1. Contribution workflow: [`CONTRIBUTING.md`](CONTRIBUTING.md)
2. Community standards: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
3. Security reporting: [`SECURITY.md`](SECURITY.md)

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).

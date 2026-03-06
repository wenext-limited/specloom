# Specloom CLI

`specloom-cli` is the CLI for running Specloom pipeline stages and agent lookup tools.

## Install from crates.io

```bash
cargo install specloom-cli
specloom --help
```

## Quickstart

Run with the installed binary:

```bash
specloom fetch --input fixture
specloom generate --input fixture
```

Run from the workspace during development:

```bash
cargo run -p specloom-cli -- fetch --input fixture
cargo run -p specloom-cli -- generate --input fixture
```

The `generate` command runs the full pipeline in order and writes artifacts under `output/`.

## End-to-End UI Generation

```bash
# 1) Run deterministic pipeline (fixture or live)
specloom generate --input fixture
specloom generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"

# 2) Build output/agent/llm_bundle.json from deterministic artifacts + docs
specloom prepare-llm-bundle --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>" --target react-tailwind --intent "Generate login screen code"

# 3) Generate target UI from the bundle
specloom generate-ui --bundle output/agent/llm_bundle.json
```

Expected outputs:

1. `output/agent/llm_bundle.json`
2. generated target code under `output/generated/<target>/...`
3. `output/reports/generation_warnings.json`
4. `output/reports/generation_trace.json`

## Live Figma Quickstart

Set your token once (or pass `--figma-token` per command):

```bash
export FIGMA_TOKEN="<YOUR_FIGMA_PERSONAL_ACCESS_TOKEN>"
```

Optional global config (plain text):

```toml
# ~/.config/specloom/config.toml
[auth]
figma_token = "..."
anthropic_api_key = "..."
```

Credential precedence: CLI flag > env var > config file.

Fetch and inspect a real node snapshot:

```bash
specloom fetch --input live --file-key <FILE_KEY> --node-id <NODE_ID>
specloom fetch --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
```

Run the full pipeline from live Figma data:

```bash
specloom generate --input live --file-key <FILE_KEY> --node-id <NODE_ID>
specloom generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
```

## Inputs and Defaults

Input modes:

1. `fixture`: uses a built-in deterministic payload for local/testing runs.
2. `live`: calls the Figma API with either `--file-key` + `--node-id`, or a single `--figma-url`.
3. `snapshot`: loads an existing raw snapshot via `--snapshot-path` and reuses it as fetch output.

Defaults:

1. `fetch` defaults to `--input fixture`.
2. `generate` defaults to `--input live`.
3. `FIGMA_TOKEN` env (or `--figma-token`) is required for `live`.
4. Downstream stages read prior artifacts from `output/`.

## Commands

List stage output directories:

```bash
specloom stages
specloom stages --output json
```

Run one stage:

```bash
specloom run-stage fetch
specloom run-stage normalize --output json
```

Run fetch stage directly (fixture, live, or snapshot):

```bash
specloom fetch
specloom fetch --input live --file-key <FILE_KEY> --node-id <NODE_ID>
specloom fetch --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
specloom fetch --input snapshot --snapshot-path <PATH_TO_FETCH_SNAPSHOT_JSON>
```

Run full pipeline:

```bash
specloom generate --input fixture
specloom generate --input fixture --output json
specloom generate --input live --file-key <FILE_KEY> --node-id <NODE_ID>
specloom generate --input live --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>"
specloom generate --input snapshot --snapshot-path <PATH_TO_FETCH_SNAPSHOT_JSON>
```

Build an LLM bundle:

```bash
specloom prepare-llm-bundle --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>" --target react-tailwind --intent "Generate login screen code"
specloom prepare-llm-bundle --figma-url "https://www.figma.com/design/<FILE_KEY>/<PAGE_NAME>?node-id=<NODE_ID>" --target swiftui --intent "Generate SwiftUI screen" --output json
```

Generate target UI from a bundle:

```bash
specloom generate-ui --bundle output/agent/llm_bundle.json
specloom generate-ui --bundle output/agent/llm_bundle.json --provider anthropic --model claude-3-5-sonnet-latest
specloom generate-ui --bundle output/agent/llm_bundle.json --output json
```

Run agent lookup tools (stateless run-and-consume):

```bash
specloom agent-tool find-nodes --query "welcome back" --output json
specloom agent-tool get-node-info --node-id <NODE_ID>
specloom agent-tool get-node-screenshot --file-key <FILE_KEY> --node-id <NODE_ID>
```

## Workflow Matrix

| Goal | Command | Output Mode |
| --- | --- | --- |
| Inspect all stage output directories | `specloom stages` | text (default) |
| Inspect all stage output directories as machine-readable data | `specloom stages --output json` | json |
| Run fetch stage with fixture input | `specloom fetch --input fixture` | text (default) |
| Run fetch stage with live Figma input | `specloom fetch --input live --file-key <file> --node-id <node>` | text (default) |
| Run fetch stage with Figma quick link input | `specloom fetch --input live --figma-url "<figma-url>"` | text (default) |
| Run fetch stage with existing snapshot artifact | `specloom fetch --input snapshot --snapshot-path <path>` | text (default) |
| Run one stage with human-readable output | `specloom run-stage <stage>` | text (default) |
| Run one stage with machine-readable output | `specloom run-stage <stage> --output json` | json |
| Run end-to-end pipeline with fixture input and per-stage artifact lines | `specloom generate --input fixture` | text (default) |
| Run end-to-end pipeline with live Figma input | `specloom generate --input live --file-key <file> --node-id <node>` | text (default) |
| Run end-to-end pipeline with Figma quick link input | `specloom generate --input live --figma-url "<figma-url>"` | text (default) |
| Run end-to-end pipeline from existing snapshot artifact | `specloom generate --input snapshot --snapshot-path <path>` | text (default) |
| Run end-to-end pipeline with fixture input and structured stage results | `specloom generate --input fixture --output json` | json |
| Build `output/agent/llm_bundle.json` from deterministic artifacts | `specloom prepare-llm-bundle --figma-url "<figma-url>" --target <target> --intent "<intent>"` | text (default) |
| Build `output/agent/llm_bundle.json` as machine-readable output | `specloom prepare-llm-bundle --figma-url "<figma-url>" --target <target> --intent "<intent>" --output json` | json |
| Generate target UI code from bundle path | `specloom generate-ui --bundle output/agent/llm_bundle.json` | text (default) |
| Generate target UI code from bundle path via Anthropic Claude | `specloom generate-ui --bundle output/agent/llm_bundle.json --provider anthropic --model claude-3-5-sonnet-latest` | text (default) |
| Generate target UI code from bundle path as machine-readable output | `specloom generate-ui --bundle output/agent/llm_bundle.json --output json` | json |
| Find candidate nodes via deterministic fuzzy lookup | `specloom agent-tool find-nodes --query "<text>" --output json` | text/json |
| Read indexed node details | `specloom agent-tool get-node-info --node-id <id>` | text/json |
| Fetch node screenshot directly from Figma images API | `specloom agent-tool get-node-screenshot --file-key <file> --node-id <node>` | text/json |

Notes:

1. Valid stages are: `fetch`, `normalize`, `build-spec`, `build-agent-context`, and `export-assets`.
2. Invalid stage execution returns exit code `2` with an explicit error message.
3. `generate` runs deterministic default stages sequentially: `fetch`, `normalize`, `build-spec`, `build-agent-context`, and `export-assets`.
4. `prepare-llm-bundle` writes `output/agent/llm_bundle.json`.
5. `generate-ui` writes generated code under `output/generated/<target>/...` and updates warning/trace reports.
6. `--provider anthropic` requires `ANTHROPIC_API_KEY` (or `--api-key`).
7. `prepare-llm-bundle` loads instruction docs from local project files first; if missing, it reads from `~/.config/specloom/release_cache/<tag>/...`.
8. When the cache is missing, `prepare-llm-bundle` downloads the matching GitHub release snapshot for the running CLI version (`v<version>`, then `<version>`), or falls back to the latest GitHub release if the current version is not yet released.
9. Downloaded instruction docs/skills are cached at `~/.config/specloom/release_cache/<tag>/...`.
10. `~/.config/specloom/config.toml` is plain text. Keep it private and never commit/upload it.
11. Agent tool commands are stateless run-and-consume invocations; no background daemon is required.

## License

Licensed under the Apache License, Version 2.0 ([../../LICENSE](../../LICENSE)).

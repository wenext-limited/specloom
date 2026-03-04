# Forge

Figma node tree to SwiftUI generator workspace in Rust 2024.

This repository implements a deterministic, stage-based pipeline that produces:

1. normalized/intermediate JSON artifacts
2. generated SwiftUI source
3. asset manifest metadata
4. review report warnings and summaries

## Quickstart

Run from the repository root:

```bash
cargo check --workspace
cargo test --workspace
cargo run -p cli -- generate
```

The `generate` command runs the full pipeline in order and writes artifacts under `output/`.

## Inputs and Outputs

Current input mode:

1. The `fetch` stage uses a built-in fixture payload (no external Figma API call yet).
2. Downstream stages read prior artifacts from `output/`.

Generated artifacts:

| Stage | Output Directory | Artifact |
| --- | --- | --- |
| `fetch` | `output/raw` | `output/raw/fetch_snapshot.json` |
| `normalize` | `output/normalized` | `output/normalized/normalized_document.json` |
| `infer-layout` | `output/inferred` | `output/inferred/layout_inference.json` |
| `build-spec` | `output/specs` | `output/specs/ui_spec.json` |
| `gen-swiftui` | `output/swift` | `output/swift/FixtureRootView.swift` |
| `export-assets` | `output/assets` | `output/assets/asset_manifest.json` |
| `report` | `output/reports` | `output/reports/review_report.json` |

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

Run full pipeline:

```bash
cargo run -p cli -- generate
cargo run -p cli -- generate --output json
```

## CLI Workflow Matrix

| Goal | Command | Output Mode |
| --- | --- | --- |
| Inspect all stage output directories | `cargo run -p cli -- stages` | text (default) |
| Inspect all stage output directories as machine-readable data | `cargo run -p cli -- stages --output json` | json |
| Run one stage with human-readable output | `cargo run -p cli -- run-stage <stage>` | text (default) |
| Run one stage with machine-readable output | `cargo run -p cli -- run-stage <stage> --output json` | json |
| Run end-to-end pipeline with per-stage artifact lines | `cargo run -p cli -- generate` | text (default) |
| Run end-to-end pipeline with structured stage results | `cargo run -p cli -- generate --output json` | json |

Notes:

1. Valid stages are: `fetch`, `normalize`, `infer-layout`, `build-spec`, `gen-swiftui`, `export-assets`, and `report`.
2. Invalid stage execution returns exit code `2` with an explicit error message.
3. `generate` runs all stages sequentially in the order listed above.

## Scope

In-scope right now:

1. deterministic stage orchestration and artifact handoff
2. warning/report surfacing for unsupported and low-confidence behavior
3. fixture-backed end-to-end generate coverage

Not yet in scope:

1. live Figma API authentication and fetch
2. full fidelity translation of advanced visual effects/interactions
3. production app architecture/accessibility/localization generation

## Verification

Use these gates before claiming completion:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/verify_workspace.sh
```

# Figma Node Tree to SwiftUI Generator Design

## Context

This design is based on [`docs/proposal.md`](/Users/wendell/Developer/forge/docs/proposal.md) and targets a Rust 2024 implementation that converts Figma node trees into deterministic, reviewable SwiftUI output.

The project is currently a fresh repository with no prior source code, so this design defines both the initial workspace structure and the execution path to reach a production-capable pipeline.

## Chosen Approach

We choose a phased workspace approach:

1. Initialize all target crates now, aligned with the proposal architecture.
2. Keep each crate minimal and compilable (no placeholder feature logic).
3. Implement behavior in staged increments with deterministic contracts first.

Why this approach:

1. Preserves the long-term architecture and crate boundaries from day one.
2. Avoids over-investing in scaffolding logic that will be replaced quickly.
3. Keeps early verification simple (`cargo check --workspace`) and reliable.

## Workspace Shape

Workspace root:

1. `Cargo.toml` (workspace members and shared metadata).
2. `crates/` (all architecture crates from proposal).
3. `output/` artifact directories for stage outputs.

Initial crates:

1. `cli` (binary crate, command entrypoint shell).
2. `figma_client` (library crate).
3. `figma_normalizer` (library crate).
4. `layout_infer` (library crate).
5. `ui_spec` (library crate).
6. `swiftui_ast` (library crate).
7. `swiftui_codegen` (library crate).
8. `asset_pipeline` (library crate).
9. `review_report` (library crate).
10. `orchestrator` (library crate).

Each library crate includes only `src/lib.rs` and `Cargo.toml`.

`cli` includes only `src/main.rs` and `Cargo.toml`.

All crates use `edition = "2024"`.

## Architecture and Data Flow

Primary execution flow:

1. `fetch`
2. `normalize`
3. `infer-layout`
4. `build-spec`
5. `gen-swiftui`
6. `export-assets`
7. `report`

Control flow:

1. `cli` parses commands and delegates stage execution.
2. `orchestrator` owns stage sequencing and typed handoff boundaries.
3. Domain crates own one concern each (fetching, normalization, inference, spec, AST, codegen, assets, report).

Persistence model:

1. Every stage reads known input artifact(s) and writes explicit output artifact(s).
2. Output directories exist up front:
   - `output/raw`
   - `output/normalized`
   - `output/inferred`
   - `output/specs`
   - `output/swift`
   - `output/assets`
   - `output/reports`

## Determinism and Contracts

Hard constraints:

1. Version metadata is mandatory in persisted artifacts.
2. Stable ordering policy is centralized and reused (especially node traversal and map-like serialization).
3. AI fallback is optional and disabled by default in the initial execution plan.

Contract focus for first implementation stages:

1. Versioned structs for normalized nodes, UI spec, SwiftUI AST, and review report.
2. Serde-based serialization with deterministic field behavior where needed.
3. Stage outputs suitable for golden/snapshot verification.

## Error Handling

Initial taxonomy:

1. Input/contract validation errors.
2. External/API or I/O failures.
3. Unsupported feature warnings (non-fatal by default).
4. Low-confidence inference warnings.

Behavior rules:

1. No silent degradation.
2. Unsupported and low-confidence paths must be reported in review artifacts.
3. CLI surfaces concise user messages while preserving rich context for logs and diagnostics.

## Testing and Verification Strategy

Milestone verification gates:

1. `cargo check --workspace` must pass after bootstrap.
2. Per-crate smoke tests ensure wiring and compileability.
3. Contract serialization round-trip tests for versioned structs.
4. Determinism checks verify byte-stable serialized outputs for repeated runs on same input.
5. One CLI integration smoke path verifies command surface wiring.

## Executable Delivery Plan (High Level)

1. Bootstrap workspace and crates with Rust 2024.
2. Add schema-first domain types in `ui_spec`, `swiftui_ast`, and `review_report`.
3. Add orchestrator stage interfaces and CLI command surface.
4. Add deterministic ordering helpers and shared error handling baseline.
5. Add initial tests and integration smoke checks.
6. Expand stage-by-stage from deterministic core to assets, reporting, and optional AI fallback.

## Approved Design Summary

The approved design balances long-term architecture fidelity with short-term execution speed:

1. Full crate topology now.
2. Minimal compilable crate internals now.
3. Deterministic contracts and verification before feature depth.
4. Explicit review/reporting model for ambiguity and unsupported features.

# Next Stage Contract Map

This document defines the next schema expansion targets after bootstrap.

## 1. Normalized Node Graph Types (`figma_normalizer`)

Add versioned types for canonical Figma data:

1. `NormalizedDocument` root with `schema_version`, source metadata, and node collection.
2. `NormalizedNode` with identity, kind, visibility, bounds, layout metadata, style, component metadata, and ordered children IDs.
3. Supporting value types for layout constraints, fills, strokes, and geometry.

## 2. Layout Decision Record (`layout_infer`)

Add explicit inference output contracts:

1. `LayoutDecisionRecord` with `decision_version`, selected strategy, confidence score, and rationale.
2. `LayoutAlternative` list for fallback strategies that were considered.
3. `InferenceWarning` list for ambiguous or low-confidence cases.

## 3. Asset Manifest Schema (`asset_pipeline`)

Define deterministic asset export contracts:

1. `AssetManifest` with `manifest_version`, generation metadata, and ordered asset entries.
2. `AssetEntry` with source node/image refs, hashed output filename, export format, dimensions, and dedupe key.
3. `AssetExportWarning` for unsupported formats or export fallback behavior.

## 4. Review Report Warning Categories (`review_report`)

Expand warnings into typed categories:

1. `ReviewWarningCategory` enum (`UnsupportedFeature`, `LowConfidenceLayout`, `FallbackApplied`, `DataLossRisk`).
2. `ReviewWarning` with stable code, category, severity, message, and source node context.
3. Report summary counters by category and severity for deterministic review output.

## 5. Phase 05 Execution Evidence (2026-03-04)

Implemented contract expansion work:

1. `P5-T1` `238365a` (`figma_normalizer`): explicit normalized schema/API version constants and expanded deterministic contract tests.
2. `P5-T2` `b119606` (`layout_infer`): strict decision/warning deserialization (`deny_unknown_fields`) and explicit contract/ordering tests.
3. `P5-T3` `8ec0dcf` (`asset_pipeline`): `generation` metadata contract and `hashed_output_filename` field with deterministic contract tests.
4. `P5-T4` `d307194` (`review_report`): deterministic summary pre-seeding for all category/severity variants and stable enum value tests.
5. `P5-T5` `c0fd3ae` (workspace verification): explicit cross-crate contract checks in `scripts/verify_workspace.sh`.

Verification evidence:

1. `cargo test -p figma_normalizer`
2. `cargo test -p layout_infer`
3. `cargo test -p asset_pipeline`
4. `cargo test -p review_report`
5. `bash scripts/verify_workspace.sh`
6. `cargo check --workspace`
7. `cargo test --workspace`

Follow-up scope for next phase:

1. Add compatibility policy notes for contract version bumps and migration testing.
2. Add fixture-based golden files for representative normalized/inferred/spec artifacts.
3. Add cross-crate integration tests that assert end-to-end contract wiring (normalizer -> infer -> report).

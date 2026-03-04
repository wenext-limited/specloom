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

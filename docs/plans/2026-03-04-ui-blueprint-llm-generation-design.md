# UI Blueprint and LLM-Driven UI Generation Design

## Context

The current pipeline in this repository is deterministic through SwiftUI code generation:

`fetch -> normalize -> infer-layout -> build-spec -> gen-swiftui -> export-assets -> report`

The approved direction is to keep deterministic Rust processing up to structured UI interpretation, then shift UI code generation to an LLM-oriented workflow.

This design defines:

1. A new LLM-friendly artifact format (`ui_blueprint.yaml`).
2. A dual-contract strategy (`ui_spec.json` + `ui_blueprint.yaml`).
3. Tooling boundaries between deterministic pipeline stages and non-deterministic LLM generation.

## User-Approved Decisions

1. Keep a hybrid architecture: deterministic Rust remains authoritative through `build-spec`.
2. Introduce a human/LLM-friendly artifact derived from current JSON outputs.
3. Use YAML for the new artifact format.
4. Keep the artifact target-agnostic (not tied to SwiftUI only).
5. Emit both artifacts:
   - `output/specs/ui_spec.json` for strict machine contracts.
   - `output/specs/ui_blueprint.yaml` for LLM consumption.
6. Use normalized high-level layout semantics in the new artifact (`stack_v`, `stack_h`, `overlay`, `absolute`, `scroll`) rather than raw Figma layout fields.
7. Rename the initial placeholder name to **UI Blueprint**.

## Goals

1. Improve LLM generation quality by removing noisy metadata from LLM input.
2. Preserve deterministic, auditable Rust stages as source-of-truth.
3. Keep downstream UI generation target-agnostic (SwiftUI, React, Compose, etc.).
4. Make stage outputs explicit, versioned, and testable.

## Non-Goals

1. Replacing deterministic normalization/inference with LLM logic.
2. Guaranteeing deterministic code output from LLM runs.
3. Removing existing `ui_spec.json` consumers in this phase.

## Approaches Considered

### 1. Dual-Contract Pipeline (Recommended and approved)

- Keep `ui_spec.json` as canonical machine contract.
- Add deterministic `build-ui-blueprint` stage for LLM-friendly YAML.
- Add separate LLM tooling to consume `ui_blueprint.yaml` and generate target UI.

Pros:
1. Lowest migration risk.
2. Strong compatibility with current pipeline/tests.
3. Clear audit boundary between deterministic and non-deterministic steps.

Cons:
1. Two artifacts require mapping maintenance.

### 2. UI Blueprint as New Canonical Contract

- Replace `ui_spec.json` with `ui_blueprint.yaml` as the primary stage output.

Pros:
1. Simpler external story.

Cons:
1. Higher migration risk.
2. Weaker typed guarantees unless additional validation tooling is introduced.

### 3. Adapter-Only Command

- Keep current stages unchanged; add a standalone converter command.

Pros:
1. Fastest initial shipping path.

Cons:
1. Easier for schema drift to occur.
2. Mapping logic can fragment across commands.

## Recommended Architecture

Primary deterministic flow:

`fetch -> normalize -> infer-layout -> build-spec -> build-ui-blueprint -> export-assets -> report`

LLM tooling flow:

1. `prepare-llm-bundle` gathers:
   - `ui_blueprint.yaml`
   - asset manifest references
   - warnings/low-confidence summary
   - target selection + prompt template
2. Optional `llm-generate-ui` executes model calls and writes generated UI output to target-specific paths.

Notes:
1. `gen-swiftui` is removed from the default happy path and treated as deprecated/optional behavior.
2. Deterministic stages remain successful even if LLM generation fails.

## Contract Design: `ui_blueprint.yaml`

Path:

- `output/specs/ui_blueprint.yaml`

Version marker:

- `version: ui_blueprint/1.0`

Top-level fields:

1. `version`
2. `document`
3. `design_tokens`
4. `components`
5. `screens`
6. `assets`
7. `warnings`

### Proposed YAML shape (illustrative)

```yaml
version: ui_blueprint/1.0
document:
  file_key: abc123
  root_node_id: 123:456
  name: Login Screen
  viewport:
    width: 390
    height: 844

design_tokens:
  colors:
    text_primary: "#1F1F1F"
    bg_surface: "#FFFFFF"
  spacing:
    s: 8
    m: 16
  radius:
    card: 12
  typography:
    title:
      size: 24
      weight: 600

components:
  - id: comp/button_primary
    name: Button Primary
    root:
      id: node:btn-root
      role: container
      layout:
        type: stack_h
        align: center
        gap: 8
      style:
        background_color: token(colors.bg_surface)
        corner_radius: token(radius.card)
      children:
        - id: node:btn-label
          role: text
          content:
            text: Continue

screens:
  - id: screen/login
    name: Login
    root:
      id: node:root
      role: container
      layout:
        type: stack_v
        gap: 16
        padding: { top: 24, right: 20, bottom: 24, left: 20 }
      children: []

assets:
  - id: asset/logo
    kind: image
    path: output/assets/logo.png

warnings:
  - code: UNSUPPORTED_BLEND_MODE
    message: Blend mode SCREEN was normalized without a 1:1 mapping.
    node_id: 123:789
```

## Mapping Rules (High Level)

1. `ui_spec.json` remains the complete typed contract.
2. `ui_blueprint.yaml` is a projection with reduced verbosity and stable ordering.
3. High-signal fields only: hierarchy, role, inferred layout, essential style, content, assets, warnings.
4. Do not include provenance-heavy metadata that does not improve generation quality.
5. Preserve warning visibility; unsupported or low-confidence areas must remain explicit.

## Layout Semantics Decision

Figma raw payload contains layout-related data (geometry, Auto Layout fields, constraints) but not universally complete high-level intent for every node.

Approved approach:

1. Use normalized and inferred high-level layout semantics in UI Blueprint.
2. Avoid exposing raw Figma layout internals as the default LLM contract.

## LLM Agent Tooling Requirements

### `prepare-llm-bundle`

1. Input: `ui_blueprint.yaml` + artifact paths.
2. Output: structured LLM bundle with concise instructions and warnings.
3. Guarantees:
   - deterministic bundle packaging
   - explicit target metadata
   - hashable inputs for audit

### `llm-generate-ui` (optional execution tool)

1. Input: prepared bundle + model config.
2. Output: generated target UI files.
3. Run metadata to persist:
   - `model`
   - `prompt_template_version`
   - `prompt_hash`
   - `input_blueprint_hash`
   - output paths

## Error Handling

1. `build-ui-blueprint` fails on required-structure violations.
2. Mapping ambiguity becomes warnings, not silent omission.
3. LLM generation errors are isolated from deterministic stage success.
4. CLI messages should separate:
   - deterministic pipeline status
   - LLM tooling status

## Verification Strategy

Unit tests:

1. UI Blueprint serde + YAML round-trip.
2. Stable YAML ordering/format snapshots.
3. `ui_spec -> ui_blueprint` mapper behavior.

Integration tests:

1. Stage execution writes both `ui_spec.json` and `ui_blueprint.yaml`.
2. Repeat runs produce byte-stable deterministic artifacts for identical input.
3. Report/warning propagation includes Blueprint output.

Workspace gates:

1. `cargo check --workspace`
2. `cargo test --workspace`

## Migration Plan (Design-Level)

1. Introduce `build-ui-blueprint` stage and artifacts.
2. Keep `ui_spec.json` unchanged for existing consumers.
3. Mark `gen-swiftui` as deprecated in default generate flow.
4. Add LLM tooling commands behind explicit invocation.
5. Maintain deterministic report/audit outputs regardless of LLM use.

## Success Criteria

1. Running the deterministic pipeline always emits:
   - `output/specs/ui_spec.json`
   - `output/specs/ui_blueprint.yaml`
2. LLM bundle preparation is reproducible and traceable.
3. Warnings remain explicit in both machine and LLM-facing contracts.
4. Existing deterministic contract consumers continue working during migration.

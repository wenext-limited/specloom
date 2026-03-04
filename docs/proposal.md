# Proposal: Figma Node Tree -> SwiftUI Generator (Rust 2024)

For current repository usage commands and artifact paths, see the root [`README.md`](/Users/wendell/Developer/forge/README.md).

## 1. Objective

Build an agent-friendly pipeline that converts **Figma node trees** into **reviewable SwiftUI code**, using **Rust 2024** for all core tooling.

Primary goals:

1. Use Figma node data, not screenshots, as the source of truth.
2. Keep AI usage limited to ambiguous layout reasoning and other low-confidence edge cases.
3. Produce deterministic, reproducible SwiftUI output from a stable intermediate spec.
4. Make every stage explicit, inspectable, and testable so an implementation agent can execute and verify it.
5. Preserve enough provenance and decision trace data that humans can audit why a given SwiftUI result was produced.

Non-goals for the first version:

1. Pixel-perfect parity for every visual effect Figma supports.
2. Full support for advanced prototyping interactions, animations, or runtime business logic.
3. Automatic generation of production-ready accessibility copy, localization content, or app architecture.
4. Perfect translation of highly artistic or marketing-heavy layouts with complex masks, blend modes, and custom vector effects.

Success means: **the pipeline can generate compileable, structurally correct SwiftUI for common product screens with deterministic output and a clear review trail**.

## 1.1 Current Implementation Update (2026-03-04)

The active pipeline has been extended with an LLM-oriented contract and tooling layer:

1. Deterministic stage `build-ui-blueprint` now emits `output/specs/ui_blueprint.yaml` in addition to `output/specs/ui_spec.json`.
2. Deterministic stage `prepare-llm-bundle` writes `output/llm/llm_bundle.json` for model consumption.
3. CLI command `generate-ui` performs direct model-based UI generation from the prepared bundle.

This keeps deterministic Rust stages authoritative while enabling target-specific generation through an explicit LLM interface.

## 2. End-to-End Architecture

```text
Figma File
   │
   ▼
Figma REST API (node tree + components + styles + image refs)
   │
   ▼
Fetch Cache / Snapshot Store
   │
   ▼
Normalizer (Rust)
   │
   ▼
Layout Inference (rules first, AI fallback)
   │
   ▼
Language-agnostic UI Spec (versioned JSON)
   │
   ▼
SwiftUI AST (Rust enums/structs)
   │
   ▼
SwiftUI Renderer (deterministic text emitter)
   │
   ├──► Generated .swift files
   ├──► Assets.xcassets
   └──► Review report / decision trace
```

Suggested artifact boundaries:

| Stage | Input | Output | Mode |
| --- | --- | --- | --- |
| Fetch | file key + node ids + token | raw Figma JSON snapshot | deterministic |
| Normalize | raw Figma JSON | canonical node graph | deterministic |
| Infer layout | canonical node graph | inferred layout annotations + confidence | rules first, optional AI |
| Build spec | inferred graph | stable UI spec JSON | deterministic |
| Generate SwiftUI | UI spec | SwiftUI AST + `.swift` files | deterministic |
| Export assets | raw refs + spec | asset files + asset manifest | deterministic |
| Report | all stage metadata | warnings / unsupported / low-confidence report | deterministic |

Responsibility split:

| Stage | Responsibility |
| --- | --- |
| Figma fetch + normalization | exact data capture and canonicalization |
| Layout inference | translate geometry + layout metadata into container semantics |
| UI spec serialization | create stable, language-agnostic contract |
| SwiftUI AST generation | map UI primitives into typed SwiftUI constructs |
| SwiftUI code rendering | emit readable, deterministic Swift source |
| Review reporting | surface ambiguity, unsupported features, and fallbacks |

## 3. Why Node Tree First

Figma nodes provide exact geometry, hierarchy, and style metadata that screenshots cannot recover reliably.

| Signal | Screenshot | Figma node tree |
| --- | --- | --- |
| Coordinates | approximate | exact |
| Typography | OCR guess | exact |
| AutoLayout direction/spacing | hidden | explicit |
| Component identity | hidden | explicit |
| Constraints/resizing | hidden | explicit |
| Fill/stroke/effects metadata | guessed | explicit |
| Variant/instance overrides | hidden | explicit |

Decision: **Figma node data is the only primary input.** Screenshot/vision may be used only as optional debugging context or for human review artifacts.

## 4. Rust 2024 Implementation Shape

Recommended workspace layout:

```text
figma-swiftui/
├ Cargo.toml
├ crates/
│  ├ cli/                 # command entrypoints
│  ├ figma_client/        # API client + auth + rate limit handling + caching
│  ├ figma_normalizer/    # flattening + canonical node model
│  ├ layout_infer/        # stack/grid/overlay/scroll inference
│  ├ ui_spec/             # stable schema structs + serde + versioning
│  ├ swiftui_ast/         # AST model
│  ├ swiftui_codegen/     # deterministic renderer
│  ├ asset_pipeline/      # export + dedupe + catalog generation
│  ├ review_report/       # warnings + unsupported feature reporting
│  └ orchestrator/        # pipeline execution + checkpoints
└ output/
   ├ raw/
   ├ normalized/
   ├ inferred/
   ├ specs/
   ├ swift/
   ├ assets/
   └ reports/
```

Key Rust choices:

1. `edition = "2024"` in all crates.
2. `serde` / `serde_json` for stable contracts.
3. `schemars` to generate JSON Schema for normalized nodes, UI spec, and AI fallback payloads.
4. `reqwest` + `tokio` for API I/O.
5. `thiserror` + `miette` for debuggable errors.
6. `tracing` for stage-level logs with file key and node IDs.
7. `indexmap` where map order needs to be preserved deterministically.
8. `insta` for snapshot tests and `proptest` for stability/property tests.

## 5. Scope of Supported Figma Features

A good proposal should be explicit about what the MVP supports versus what it only reports.

### 5.1 MVP support target

The first implementation should aim to support:

1. `FRAME`, `GROUP`, `COMPONENT`, `INSTANCE`, `COMPONENT_SET`, `TEXT`, `RECTANGLE`, `ELLIPSE`, `VECTOR`, and image fills.
2. AutoLayout direction, spacing, padding, alignment, hugging/fill behavior, and basic constraints.
3. Background fills, corner radius, stroke, opacity, shadows where SwiftUI has a straightforward equivalent.
4. Text content and text style attributes needed for common app UI.
5. Component extraction and instance overrides for repeated reusable subtrees.
6. Absolute positioning only when it is necessary and reviewable.

### 5.2 Partial or explicit non-support in MVP

The first version should detect and report, but not promise full fidelity for:

1. Complex blend modes and masks.
2. Advanced boolean/path effects and highly custom vector art.
3. Prototype flows, interactions, and animation timing.
4. Video, embedded content, and third-party widgets.
5. Nontrivial variable font behavior or text-on-path.
6. Effects that do not map cleanly to stock SwiftUI APIs.

Rule: **unsupported or partially supported features must be emitted into the review report, never silently dropped without notice**.

## 6. Agent-Usable Data Contracts

The agent should rely on strict, versioned schemas to avoid prompt drift and to keep later stages stable even if implementation details evolve.

### 6.1 Canonical Figma Node (normalized)

Example shape:

```json
{
  "schema_version": "1.0",
  "source": {
    "file_key": "abc123",
    "node_id": "123:456",
    "figma_api_version": "v1"
  },
  "id": "123:456",
  "parent_id": null,
  "name": "LoginPanel",
  "kind": "frame",
  "visible": true,
  "bounds": { "x": 0.0, "y": 0.0, "w": 375.0, "h": 812.0 },
  "layout": {
    "mode": "vertical",
    "primary_align": "start",
    "cross_align": "stretch",
    "item_spacing": 20.0,
    "padding": { "top": 24.0, "right": 24.0, "bottom": 24.0, "left": 24.0 }
  },
  "constraints": {
    "horizontal": "stretch",
    "vertical": "min"
  },
  "style": {
    "opacity": 1.0,
    "corner_radius": 16.0,
    "fills": [],
    "strokes": []
  },
  "component": {
    "component_id": null,
    "component_set_id": null,
    "instance_of": null,
    "variant_properties": {}
  },
  "children": ["123:457", "123:458", "123:459"]
}
```

### 6.2 UI Spec (stable target)

Example shape:

```json
{
  "spec_version": "1.0",
  "source": {
    "file_key": "abc123",
    "root_node_id": "123:456",
    "generator_version": "0.1.0"
  },
  "type": "panel",
  "name": "LoginPanel",
  "layout": {
    "kind": "vstack",
    "spacing": 20,
    "padding": [24, 24, 24, 24],
    "alignment": "leading"
  },
  "style": {
    "background": null,
    "corner_radius": 16
  },
  "children": [
    {
      "type": "text",
      "value": "Login",
      "font": { "size": 28, "weight": "bold" }
    },
    {
      "type": "text_field",
      "placeholder": "Email"
    },
    {
      "type": "button",
      "title": "Sign In"
    }
  ],
  "review": {
    "min_confidence": 0.93,
    "warnings": []
  }
}
```

Contract rules:

1. Every persisted schema must carry a version field.
2. The UI spec must remain backward-compatible across generator revisions whenever possible.
3. Field ordering in serialized JSON should be stable.
4. Unknown fields should be ignored safely by downstream readers when feasible.
5. Provenance metadata should make it possible to trace generated code back to the source Figma node tree and generator version.

## 7. Layout Inference Strategy

Inference is the hardest step, so agent behavior must be explicit and explainable.

### 7.1 Rule-first mapping

Baseline mappings:

1. Vertical AutoLayout -> `VStack`
2. Horizontal AutoLayout -> `HStack`
3. Overlapping bounds ratio above threshold -> `ZStack`
4. 2D regular matrix with stable cell sizes -> `Grid`
5. Content taller than viewport + clipping hints -> `ScrollView(.vertical)`
6. Content wider than viewport + clipping hints -> `ScrollView(.horizontal)`
7. Single child expanded by hugging/fill rules -> spacer/frame inference
8. Repeated children with same source component -> candidate reusable view / list-like structure

Additional heuristics the proposal should make explicit:

1. Sort sibling nodes by `y` then `x` before heuristic evaluation unless Figma AutoLayout order is authoritative.
2. Treat tiny position deltas under an epsilon threshold as alignment noise, not semantic layout changes.
3. Infer `Spacer()` only when a gap is best explained by fill behavior rather than fixed spacing.
4. Prefer stacks over absolute positioning when both produce similar visual results.
5. Only emit absolute `.position` / `.offset` for genuinely freeform layouts or overlays.

### 7.2 Confidence + explainability

Each inference should emit a decision record:

```json
{
  "node_id": "123:456",
  "chosen_layout": "vstack",
  "confidence": 0.93,
  "signals": [
    "layout_mode=vertical",
    "uniform_spacing=20",
    "x_alignment=leading"
  ],
  "alternatives": [
    { "layout": "zstack", "score": 0.12 }
  ]
}
```

This record is not just debug output; it is part of the review surface.

### 7.3 AI fallback (only when low confidence)

Trigger AI only when rule confidence `< 0.70`.

Guardrails:

1. The model never receives the entire file if a subtree is enough.
2. The model input should be normalized JSON plus strict schema instructions, not arbitrary prose.
3. The model must return structured JSON matching a validated schema; otherwise reject it.
4. AI output should augment the decision record with structured signals, not replace provenance.
5. The final pipeline result should remain deterministic for a fixed model response fixture in tests.

### 7.4 Manual review gate

If any of the following happen, emit a high-visibility warning in the report:

1. confidence `< 0.40`
2. unsupported feature present in the subtree
3. multiple layout candidates have near-equal scores
4. absolute positioning was chosen for a large subtree

## 8. Component, Variant, and Override Strategy

This is one of the biggest practical gaps in many design-to-code systems and should be first-class in the proposal.

Rules:

1. Preserve Figma component identity during normalization.
2. Emit one SwiftUI view per reusable component when reuse count or complexity crosses a threshold.
3. Preserve instance overrides as data in the UI spec rather than flattening them away too early.
4. Map simple variants to SwiftUI parameters or enums.
5. Avoid extracting tiny one-off wrappers that make generated code harder to read.

Example mapping:

- Figma component set `Button / {kind=primary|secondary, size=sm|lg}`
- SwiftUI output could become `ButtonView(kind:size:title:)`
- Instance text/icon overrides remain instance-level inputs, not duplicated component bodies

Decision rule: **prefer semantic component reuse over purely visual subtree deduplication when both are available**.

## 9. SwiftUI AST + Codegen Rules

Generate an AST before text to preserve consistency and testability.

AST example:

```json
{
  "node": "VStack",
  "spacing": 20,
  "alignment": "leading",
  "children": [
    {
      "node": "Text",
      "value": "Login",
      "modifiers": [
        "font(.system(size: 28, weight: .bold))"
      ]
    },
    { "node": "TextField", "placeholder": "Email" },
    { "node": "Button", "title": "Sign In" }
  ]
}
```

Codegen constraints:

1. Deterministic node ordering with stable sort by `y` then `x` where required.
2. Consistent indentation, line breaks, and modifier ordering.
3. No random IDs or timestamped names in generated code.
4. Isolate reusable components into separate files.
5. Prefer explicit SwiftUI types over `AnyView`.
6. Keep rendering deterministic even when optional features are disabled.

Recommended canonical modifier order:

1. content-specific modifiers (`font`, `foregroundStyle`, `multilineTextAlignment`)
2. sizing (`frame`, `fixedSize`)
3. spacing/padding
4. background / overlay
5. clipping / corner radius / mask equivalents
6. shadows / opacity
7. accessibility modifiers

Other practical rules:

1. Preserve text literals separately from future localization decisions.
2. Generate minimal wrappers so the result stays readable to humans.
3. Emit compileable code before attempting stylistic optimization.
4. Gate newer SwiftUI APIs behind a configurable deployment target.
5. Optionally run generated code through `swift-format` in CI, but the raw emitter should already be stable.

## 10. Styling, Accessibility, and Localization Considerations

The proposal should acknowledge these explicitly even if the first version handles them conservatively.

### 10.1 Styling

The normalized model and UI spec should retain enough data to support:

1. fills / background colors
2. borders / strokes
3. corner radius
4. opacity
5. shadow where equivalent
6. image content mode when inferable

### 10.2 Accessibility

The generator should not claim full accessibility automation, but it should:

1. preserve semantic control types (`Button`, `TextField`, `Toggle`, etc.)
2. emit accessibility warnings when a tappable region has no obvious label
3. avoid converting everything into generic `ZStack` + `onTapGesture`
4. optionally include placeholder accessibility identifiers for repeated components

### 10.3 Localization

Text should be preserved in a way that makes later localization possible.

MVP recommendation:

1. keep raw text values in the spec
2. optionally emit a sidecar text manifest for later localization extraction
3. do not hardcode a localization pipeline into the first implementation

## 11. Asset Pipeline

1. Export referenced image fills and vectors from Figma.
2. Decide vector handling policy explicitly: preserve as PDF/SVG asset when possible, rasterize only when required.
3. Compute SHA-256 hash for content deduplication.
4. Rename to deterministic IDs or semantic names when available.
5. Emit `Assets.xcassets` structure plus `Contents.json`.
6. Produce an asset manifest linking generated Swift references back to original Figma nodes and hashes.

Output example:

```text
output/assets/Assets.xcassets/
├ login_bg.imageset/
├ icon_mail.imageset/
└ logo_mark.imageset/
```

Asset manifest example:

```json
{
  "node_id": "123:700",
  "asset_name": "icon_mail",
  "sha256": "...",
  "kind": "pdf",
  "source_ref": "imageRef:xyz"
}
```

## 12. Review Report and Unsupported Feature Handling

A generated review artifact will make the system much more useful for humans and agents.

For every run, emit a report such as:

```json
{
  "root_node_id": "123:456",
  "warnings": [
    "low_confidence_layout: 123:480",
    "unsupported_mask_effect: 123:512"
  ],
  "unsupported_nodes": [
    { "id": "123:512", "reason": "mask with complex blend mode" }
  ],
  "component_extractions": [
    { "id": "123:900", "swift_name": "PrimaryButton" }
  ],
  "confidence_summary": {
    "min": 0.42,
    "avg": 0.88
  }
}
```

Rule: **no silent degradation**. If fidelity drops, the report must say where and why.

## 13. CLI Workflow (for Humans + Agents)

```bash
cargo run -p cli -- fetch --file-key <KEY> --node-id <ID> --out output/raw/login.json
cargo run -p cli -- normalize --input output/raw/login.json --out output/normalized/login.json
cargo run -p cli -- infer-layout --input output/normalized/login.json --out output/inferred/login.json
cargo run -p cli -- build-spec --input output/inferred/login.json --out output/specs/login_panel.json
cargo run -p cli -- gen-swiftui --spec output/specs/login_panel.json --out output/swift/
cargo run -p cli -- export-assets --spec output/specs/login_panel.json --out output/assets
cargo run -p cli -- report --spec output/specs/login_panel.json --out output/reports/login_panel.json
```

Each command should support:

1. `--dry-run`
2. `--verbose`
3. `--fail-on-warning`
4. `--config <PATH>`

Recommended config file concerns:

1. Figma auth token source
2. output paths
3. layout thresholds
4. AI fallback enablement
5. deployment target and SwiftUI API profile
6. component extraction thresholds

## 14. Agent Execution Plan (Concrete)

Use these checkpoints so an implementation agent can work reliably.

1. **Schema first**  
   Define normalized node structs, UI spec structs, review report structs, and AST structs. Generate JSON Schemas and add serde round-trip tests.
2. **Build deterministic core**  
   Implement fetch -> normalize -> infer -> spec transform with AI disabled.
3. **Add component handling**  
   Preserve component identity, variant metadata, and instance overrides. Add extraction thresholds and naming rules.
4. **Add SwiftUI rendering**  
   Generate valid Swift syntax from AST and snapshot-test outputs.
5. **Add assets pipeline**  
   Export, dedupe, catalog, and emit asset manifest.
6. **Add review reporting**  
   Emit warnings, unsupported feature summaries, and confidence traces.
7. **Introduce AI fallback**  
   Guard behind config flag and strict schema validation.
8. **Production hardening**  
   Retry/rate-limit handling, fetch caching, structured logs, and clear error taxonomy.

Done criteria per stage:

1. Input/output JSON fixtures pass in CI.
2. Generated Swift compiles in a sample Xcode project.
3. Re-running the pipeline on the same input produces byte-identical output.
4. Warnings and unsupported features appear in report artifacts deterministically.
5. AI fallback can be disabled without breaking the rest of the pipeline.

## 15. Testing Strategy

1. Unit tests for geometry, spacing, alignment, and container heuristics.
2. Snapshot tests for normalized output, UI spec, review reports, and SwiftUI code.
3. Golden-file tests on representative Figma screens.
4. Property tests for ordering stability and naming determinism.
5. Integration test: run the full pipeline from fixture node tree to `.swift` + assets + report.
6. Compile test: build generated Swift in a sample project in CI.
7. Performance test: large-screen / large-file fixture to catch pathological slowdowns.

Recommended fixture categories:

1. simple form screen
2. settings list
3. card feed
4. tab bar shell
5. modal sheet
6. marketing-heavy edge case
7. component-set / variant-heavy screen

## 16. Operational Concerns

### 16.1 Auth and secrets

1. Figma tokens should come from environment variables or config indirection, never hardcoded.
2. Logs must not print secret values.
3. Stored snapshots should avoid including unnecessary secret-bearing request metadata.

### 16.2 Caching and retries

1. Cache raw Figma responses by file key + node id + revision where possible.
2. Retry transient API failures with backoff.
3. Respect rate limits explicitly in the client crate.

### 16.3 Provenance

Generated artifacts should include enough metadata to answer:

- what Figma subtree produced this file?
- which generator version produced it?
- which layout decisions were low confidence?

A lightweight header comment in generated Swift can help, as long as it stays deterministic.

## 17. Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Ambiguous mixed layouts | wrong UI hierarchy | confidence scoring + fallback + review report |
| Figma API changes | parser breakage | isolate API adapter crate + contract tests |
| Excessive generated file size | poor maintainability | component extraction thresholds |
| Non-determinism | noisy diffs | stable ordering + deterministic names + snapshot tests |
| Variant explosion | unreadable generated API | parameter caps + enum mapping strategy |
| Unsupported visual effects | false confidence | explicit report output + warning gates |
| Large files / rate limits | slow or failed runs | caching + pagination + backoff |

## 18. Realistic Quality Expectations

| UI class | Expected fidelity | Notes |
| --- | --- | --- |
| Simple forms/settings | 85-95% | strongest fit for deterministic rules |
| Typical app screens | 75-90% | good candidate for limited manual polish |
| Component-heavy product UI | 75-90% | depends on variant handling quality |
| Marketing-heavy / complex visual effects | 55-75% | should emit more warnings and review flags |

Human polish is still expected for final production visuals, accessibility refinement, and app-specific behavior.

## 19. Strategic Value

If implemented with the constraints above, this system can reduce UI implementation effort by roughly **50-70%** on design-heavy flows while keeping outputs reviewable, reproducible, and suitable for agent-assisted iteration.

The most important design principle is: **optimize for deterministic structure first, then controlled fidelity improvements**. That keeps the system useful even when perfect visual parity is not possible.

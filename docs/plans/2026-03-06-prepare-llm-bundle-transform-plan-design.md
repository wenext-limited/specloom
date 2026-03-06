# Prepare LLM Bundle Transform Plan Design

## Goal

Guarantee that `prepare-llm-bundle` never packages a stale or empty semantic transform state.

## Problem

The current workflow claims `transform_plan.json` is authoritative for final `ui_spec.ron`, but the mainline implementation allows `build-spec` to write an empty default plan and continue. That means `prepare-llm-bundle` can snapshot a pre-transform or effectively untransformed `ui_spec.ron`, which weakens downstream generation quality.

## Decision

Make `prepare-llm-bundle` the transform-readiness gate.

Before it emits `output/agent/llm_bundle.json`, it must:

1. Read the current pre-layout tree.
2. Reuse an existing non-empty transform plan only if it validates cleanly.
3. Author a non-empty transform plan when the existing plan is missing or empty.
4. Re-run `build-spec`.
5. Re-run `build-agent-context`.
6. Bundle only the refreshed transformed artifacts.

## Authoring Strategy

The repository does not yet have a full agent-backed transform authoring runtime in mainline code, so v1 uses deterministic heuristics:

1. Always author at least one explicit root decision.
2. Infer `HStack`, `VStack`, `ZStack`, and `ScrollView` from normalized child bounds.
3. Preserve children with `child_policy.mode = "keep"` for authored layout decisions.
4. Never overwrite a non-empty user-authored plan with heuristics.
5. Fail on invalid non-empty plans instead of silently replacing them.

## Why Here

`prepare-llm-bundle` is the right operator checkpoint because it already defines the boundary between deterministic artifacts and generation-time agent work. Folding transform readiness into that command preserves a simple operator flow:

`generate -> prepare-llm-bundle -> generate-ui`

## Verification

1. Missing or empty transform plan triggers authored decisions.
2. Existing non-empty transform plan is preserved.
3. Refreshed `ui_spec.ron` and agent context reflect the final authored plan.

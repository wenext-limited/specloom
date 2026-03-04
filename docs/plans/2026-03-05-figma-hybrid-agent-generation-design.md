# Figma Hybrid Screenshot + Node-Agent Generation Design

## Context

This design refines the repository pipeline for an agentic UI generation workflow where:

1. A screen screenshot is used for visual grounding.
2. Figma node data remains authoritative for structured UI details.
3. The LLM acts as a tool-using agent that fetches node details on demand.

The design explicitly targets the current Rust workspace and keeps deterministic stages authoritative.

## Decisions Confirmed

The following decisions were validated during design review on 2026-03-05:

1. Source of truth model: strict hybrid (screenshot + Figma nodes both required).
2. Mismatch policy: warn and continue (no hard-fail in v1).
3. Screenshot source in v1: Figma node screenshots only.
4. Target output in v1: target chosen at generation time by user.
5. Agent topology in v1: single-agent first (sub-agent mode deferred).
6. Data format: JSON contracts (not YAML/TOML for primary artifacts).
7. Search: support fuzzy lookup from visual/text context.
8. CLI runtime model: stateless run-and-consume in v1 (no always-on background process).

## Goals

1. Improve agent generation quality without flooding context with full Figma JSON.
2. Make text and non-text element lookup reliable through deterministic search tooling.
3. Preserve auditability with structured warnings and trace artifacts.
4. Keep output target-agnostic so runtime target selection is possible.

## Non-Goals (v1)

1. Full multi-agent orchestration by default.
2. External screenshot matching (outside Figma file scope).
3. Silent auto-correction on screenshot/node mismatch.
4. Replacing deterministic core stages with direct LLM-only generation.
5. Requiring a daemon/session server for normal v1 operation.

## Architecture

Deterministic core stays authoritative:

1. `fetch`
2. `normalize`
3. `infer-layout`
4. `build-spec`

New deterministic prep for agent generation:

1. `build-agent-context` (new stage)
2. `prepare-llm-bundle` (existing/compatible role)
3. `generate-ui` (single agent, tool-calling)

High-level flow:

```text
Figma -> fetch -> normalize -> infer-layout -> build-spec
      -> build-agent-context -> prepare-llm-bundle -> generate-ui(target)
                                                    -> output/generated/<target>/*
                                                    -> output/reports/*
```

## CLI Execution Model

Execution model for v1:

1. Run-and-consume by default.
2. Each CLI/tool command reads deterministic artifacts, performs work, writes outputs, and exits.
3. No persistent background service is required to use lookup/generation flows.

Implications:

1. Behavior is easier to test and replay from artifacts.
2. Failures are isolated to one invocation.
3. Tool commands stay composable in scripts/agents.

Future extension (out of scope for v1):

1. Optional long-lived session mode (for performance only), while preserving artifact-compatible behavior.

## Artifact Contracts (JSON)

### 1) `output/agent/agent_context.json`

Purpose: lightweight startup brief for the agent.

Contents:

1. Root screen metadata and screenshot references.
2. Compact skeleton tree with stable node identifiers.
3. Tool contract version and generation rules.
4. Mismatch policy (`warn_and_continue`).

Example:

```json
{
  "version": "agent_context/1.0",
  "screen": {
    "root_node_id": "1:2",
    "root_screenshot_ref": "output/images/root_1_2.png"
  },
  "rules": {
    "on_node_mismatch": "warn_and_continue"
  },
  "tools": ["find_nodes", "get_node_info", "get_node_screenshot", "get_asset"],
  "skeleton": [
    { "node_id": "1:10", "type": "FRAME", "name": "Header", "path": "Main/Header" }
  ]
}
```

### 2) `output/agent/search_index.json`

Purpose: deterministic fuzzy-search index for node retrieval.

Per-node indexed fields:

1. `node_id`, `name`, `type`, hierarchy path.
2. Visible text tokens.
3. Normalized tokens (lowercase, punctuation stripped, optional stemming).
4. Name aliases and optional OCR-style token variants.
5. Geometry tags (for example `header`, `footer`, `left_col`, `center`).

### 3) `output/reports/generation_warnings.json`

Purpose: durable warning record for mismatch and ambiguity.

Warning types:

1. `NODE_NOT_FOUND`
2. `LOW_CONFIDENCE_MATCH`
3. `MULTIPLE_CANDIDATES`
4. `SCREENSHOT_NODE_MISMATCH`
5. `UNSUPPORTED_STYLE_MAPPING`

Required warning fields:

1. `warning_id`
2. `severity`
3. `node_query`
4. `candidate_node_ids`
5. `agent_action`
6. `message`

### 4) `output/reports/generation_trace.json`

Purpose: audit trace for agent behavior and tool usage.

Contains:

1. Ordered tool calls and responses.
2. Candidate rankings returned by search.
3. Final selected node IDs per generated code segment.

## Tool API Contract

All tools must return explicit status values and never silent null behavior.

### `find_nodes(query, top_k, filters)`

Returns:

1. `status`: `ok | no_match | ambiguous`
2. Ordered `candidates[]` with:
   - `node_id`
   - `score` (`0.0..1.0`)
   - `match_reasons[]`

### `get_node_info(node_id)`

Returns:

1. `status: ok` with style/layout/token-resolved payload; or
2. `status: not_found` with machine-readable reason.

### `get_node_screenshot(node_id)`

Returns:

1. `status: ok` with image reference; or
2. `status: not_found`.

### `get_asset(node_id, format)`

Returns:

1. `status: ok` with exported asset ref/path; or
2. `status: not_found | unsupported`.

## Deterministic Fuzzy Ranking

Rust-side ranking is deterministic; the LLM does not compute ranking.

Default weighted score:

1. Text token overlap: `0.45`
2. Node name alias match: `0.20`
3. Hierarchy path similarity: `0.20`
4. Geometry hint match: `0.15`

Tie-break:

1. Sort by `(score desc, node_id asc)`.

Thresholds:

1. `>= 0.72`: confident match.
2. `0.45 - 0.72`: low-confidence match, warn.
3. `< 0.45`: no match, warn and continue.

## Generation Behavior

Runtime behavior for v1:

1. Ask user target at generation start (for example `swiftui`, `react-tailwind`).
2. Agent and tool interactions run in stateless command calls over persisted artifacts.
3. Agent builds code section-by-section using tool calls.
4. On not-found or ambiguity, agent must continue best-effort output.
5. All mismatch/ambiguity outcomes are recorded in warning artifacts.

This preserves throughput while making uncertainty explicit.

## Agent Playbook

Add one markdown playbook to standardize behavior:

1. File: `docs/agent-playbook.md` (planned implementation artifact).
2. Defines required tool call order:
   - `find_nodes`
   - `get_node_info`
   - `get_node_screenshot` (for disambiguation)
   - `get_asset` (for visual assets)
3. Defines warning policy and output schema compliance.
4. Defines "no silent fallback" rules.

v1 execution mode:

1. Default single-agent mode only.
2. Multi-agent mode is a future opt-in extension once baseline quality is stable.

## Error Handling and Reporting

Policy:

1. Deterministic stage errors remain explicit and fail-fast where artifacts are missing.
2. Generation mismatches are non-fatal by default and produce warnings.
3. Tool failures must include structured error reasons.

Reviewability:

1. Every warning must link back to query text and candidate IDs.
2. Every generated target output must have a matching trace file.

## Testing Strategy

1. Unit tests:
   - token normalization behavior
   - ranking score determinism
   - tie-break stability
2. Contract tests:
   - strict serde round-trip for `agent_context.json`
   - warnings schema validation
3. Integration tests:
   - end-to-end single-agent run with known fixture
   - mismatch scenario producing `NODE_NOT_FOUND`
   - low-confidence scenario producing `LOW_CONFIDENCE_MATCH`
4. Regression tests:
   - repeated run emits byte-stable search ranking output for same input.

## Rollout Plan (High-Level)

1. Add `build-agent-context` stage and contracts.
2. Add deterministic search index builder and `find_nodes` API.
3. Add screenshot-by-node tool integration.
4. Add warning/trace writers in generation stage.
5. Add single-agent playbook and target selection prompt wiring.

## Success Criteria

1. Agent can generate target UI code from Figma-rooted screenshot + node tools without full raw tree in prompt.
2. Text and text-less elements are recoverable via deterministic fuzzy lookup + disambiguation tools.
3. Node mismatches never silently disappear; warnings are always emitted.
4. Same inputs produce stable ranking output and reproducible reports.

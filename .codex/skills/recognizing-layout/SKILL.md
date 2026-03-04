---
name: recognizing-layout
description: Use when deciding container layout strategy from Figma or normalized node geometry and you need explicit confidence, alternatives, and warnings.
---

# Recognizing Layout

## Overview

Use this skill to classify container layout as `v_stack`, `h_stack`, `overlay`, `absolute`, or `scroll` using node-tree evidence.
Do not rely on node names or intuition alone; decisions must be traceable.

## When to Use

- You are converting Figma node trees into deterministic layout decisions.
- You need confidence scoring and fallback alternatives.
- You must surface ambiguous or unsupported structure as warnings.

## Workflow

1. Build structural evidence.
- Gather child order, bounds, overlap ratio, spacing deltas, alignment consistency, and constraints.
- Treat explicit Auto Layout metadata as the strongest signal.

2. Score layout candidates.
- `v_stack`: top-to-bottom order, stable vertical gaps, aligned x anchor.
- `h_stack`: left-to-right order, stable horizontal gaps, aligned y anchor.
- `overlay`: meaningful sibling overlap and shared anchor/center.
- `absolute`: mixed anchors and weak flow consistency.
- `scroll`: flow exists but content span exceeds viewport on one axis.

3. Compute confidence.
- Increase confidence for consistent ordering, spacing, and alignment.
- Decrease confidence for mixed signals, outliers, and contradictory overlap.
- If confidence `< 0.75`, emit a low-confidence warning.

4. Record alternatives and warnings.
- Keep top 2 alternatives with rationale.
- Emit warnings for unsupported features and potential data-loss risk.

## Required Output Shape

```json
{
  "selected_strategy": "v_stack",
  "confidence": 0.88,
  "rationale": "children are ordered top-to-bottom with stable spacing",
  "alternatives": [
    {
      "strategy": "overlay",
      "score": 0.31,
      "rationale": "minor overlap from decorative badge"
    }
  ],
  "warnings": []
}
```

## Common Mistakes

- Picking strategy from node type alone.
- Treating any overlap as `overlay`.
- Omitting alternatives for low-confidence decisions.
- Hiding ambiguous/unsupported behavior instead of warning.

## Quick Start Prompt

`Use $recognizing-layout to classify layout for <node set>, including confidence, alternatives, and warnings.`

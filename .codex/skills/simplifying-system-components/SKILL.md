---
name: simplifying-system-components
description: Use when system chrome appears and should be flattened into minimal semantic structure instead of preserving decorative internals.
---

# Simplifying System Components

## Use For

Status bars, home indicators, signal/battery clusters, and similar OS chrome.

## Rules

1. Preserve semantics, drop decoration.
2. Prefer simplified `HStack` or `Container` shells.
3. Use `replace_with` for essential children only.
4. Use `drop` for decorative internals.
5. Keep replacement child order stable.
6. Make rationale explicit (example: `flattened system component: iOS status bar`).

## Quick Check

1. Many tiny vector/shape nodes?
2. No product-level semantics in internals?
3. Path/name indicates system chrome?

If all true, simplify.

## Red Flag

Do not flatten normal product UI just because it is visually dense.

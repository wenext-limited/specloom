---
name: inferring-repeat-element-ids
description: Use when deciding whether transform decisions should include repeat_element_ids metadata from clear repeated-structure evidence.
---

# Inferring repeat_element_ids

## Meaning

1. `repeat_element_ids` describes repeated instances of the current node.
2. It is not child selection or replacement.

## Set Only When Clear

1. Repeated row/card/item structure is explicit.
2. Hierarchy and geometry support repetition.
3. Repetition semantics are stable across instances.

## Skip When Ambiguous

1. Decorative duplication only.
2. Weak or conflicting evidence.
3. Risk of semantic drift.

## Rules

1. IDs must be unique.
2. IDs must be stably ordered.
3. If uncertain, omit and explain uncertainty in `reason`.
4. When repeating the current node itself, REMOVE the rest repeated nodes from the `children` list.

## Red Flag

Do not force repeat metadata to make downstream codegen "look cleaner".

# Project Skills Guide

This file defines the project-local skills and how agents should use them.

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use when deciding container layout strategy (`v_stack`, `h_stack`, `overlay`, `absolute`, `scroll`) from node geometry and metadata.

2. `planning-implementation-work`
Path: `.codex/skills/planning-implementation-work/SKILL.md`
Use when converting approved scope into a phased implementation plan with dependencies, verification gates, and commit boundaries.

3. `parallel-phase-workflow`
Path: `.codex/skills/parallel-phase-workflow/SKILL.md`
Use when executing or maintaining phase boards with `[ ]`, `[~]`, `[x]` status and merge-to-main transitions.

## Usage Order

1. Run `recognizing-layout` first when layout decisions are needed.
2. Run `planning-implementation-work` after scope is approved.
3. Run `parallel-phase-workflow` while executing plan boards.

## Prompt Patterns

- `Use $recognizing-layout to classify layout for <node set> and output confidence + warnings.`
- `Use $planning-implementation-work to produce a phased plan and boards for <approved scope>.`
- `Use $parallel-phase-workflow to execute the active board and update task status with verification evidence.`

## Rules

1. Do not mix layout inference and implementation planning in one skill.
2. Keep warnings explicit for low-confidence or unsupported cases.
3. Every planned task must include a verification command.

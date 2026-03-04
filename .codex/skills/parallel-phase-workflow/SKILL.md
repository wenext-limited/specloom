---
name: parallel-phase-workflow
description: Coordinate phase-based multi-agent delivery using a shared board with `[ ]`, `[~]`, and `[x]` task states, dependency-aware claiming, and merge-to-main transitions. Use when splitting work across agents, creating/updating `docs/plans/boards/*.md`, running a phase from active board tasks, or closing a phase and starting the next one from latest `main`.
---

# Parallel Phase Workflow

## Overview

Use this skill to run implementation as explicit phase boards that are safe for parallel agent execution.
In this repository, use these files first:

- `docs/plans/boards/README.md`
- `docs/plans/templates/parallel-phase-board-template.md`
- Active board in `docs/plans/boards/`

## Status Model

- `[ ]` not started
- `[~]` in progress (claimed by one owner)
- `[x]` completed and merged to current phase branch/main

## Standard Workflow

1. Create or refresh a board from the template.
2. Split work into small tasks with explicit dependencies and file ownership.
3. Mark only dependency-ready tasks as claimable.
4. Claim one task by changing `[ ]` to `[~]`, setting owner, and adding start note.
5. Implement with tests and commit.
6. Update the same row to `[x]` with commit hash and verification evidence.
7. Repeat until all tasks are `[x]`.
8. Run full verification (`cargo check --workspace`, `cargo test --workspace`).
9. Merge completed phase into `main` and verify again on merged `main`.
10. Start the next phase board from latest `main`.

## Board Requirements

Each board row must include:

- `Status`, `ID`, `Task`, `Owner`, `Depends On`, `Files`, `Verification`, `Commit`, `Notes`

Use deterministic task IDs, for example `P4-T3`.

## Parallel Dispatch Rules

1. Dispatch in parallel only when owned files do not overlap.
2. Keep one owner per `[~]` task.
3. Do not claim tasks with unmet dependencies.
4. Include task ID, owned files, done criteria, and verification command in each subagent assignment.
5. If blocked, keep `[~]` and write blocker details in `Notes`.
6. If relinquishing task, set status back to `[ ]`, clear owner, keep blocker context.

## Completion Rules

1. Do not mark `[x]` without a commit and passing task verification.
2. Do not close phase until all rows are `[x]` and workspace checks pass.
3. Do not start next phase from old phase branches; always branch from latest `main`.

## Quick Start Prompt

Use this instruction pattern when invoking the skill:

`Use $parallel-phase-workflow to run the current board, claim the next dependency-ready task, execute it with verification, and update status/commit evidence.`

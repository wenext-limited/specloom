---
name: planning-implementation-work
description: Use when turning approved scope into an executable implementation plan with phased tasks, dependencies, verification gates, and commit boundaries.
---

# Planning Implementation Work

## Overview

Use this skill to convert approved requirements into a deterministic execution plan and phase boards.
The output must be specific enough for another agent to execute without hidden assumptions.

**REQUIRED SUB-SKILL:** Use superpowers:writing-plans for implementation plan structure.
**REQUIRED SUB-SKILL:** Use parallel-phase-workflow when work can be parallelized.

## When to Use

- A design/spec exists and implementation is multi-step.
- Multiple crates/components are involved.
- You need explicit dependencies, ownership, and verification gates.

## Workflow

1. Define outcome and done criteria.
- One-sentence goal.
- Concrete deliverables.
- Required verification commands.

2. Build dependency graph.
- Identify tasks that must be sequential.
- Mark tasks that can run in parallel without file overlap.

3. Write task-level execution steps.
- One logical change per task.
- Include files, behavior delta, verification command, commit message.
- Keep tasks small and independently verifiable.

4. Group tasks into phases.
- Typical shape: contracts -> stage logic -> orchestration/CLI -> e2e -> docs/hardening.
- Ensure each phase has explicit exit criteria.

5. Enforce verification gates.
- Task gate: targeted crate tests.
- Phase gate: `cargo check --workspace`, `cargo test --workspace`, repo verification script.
- Merge phase only when every board row is `[x]`.

## Required Outputs

1. Implementation plan file in `docs/plans/YYYY-MM-DD-<topic>.md`
2. Phase board file(s) in `docs/plans/boards/YYYY-MM-DD-<phase>-board.md`

Each board row must include:

- status
- task id
- owner
- dependencies
- files
- verification command
- commit hash

## Common Mistakes

- Tasks too large to verify quickly.
- Missing dependency annotations.
- Verification commands not aligned with changed files.
- Closing a phase without merged-`main` verification.

## Quick Start Prompt

`Use $planning-implementation-work to turn <approved scope> into a phased implementation plan with board tasks, dependencies, and verification gates.`

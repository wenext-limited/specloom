# <Phase Name> Parallel Board

**Phase ID:** `<phase-id>`
**Goal:** `<one-sentence outcome>`
**Source Plan:** `<path to design/implementation plan>`
**Last Updated:** `<YYYY-MM-DD HH:MM TZ>`

## Status Legend

- `[ ]` not started
- `[~]` in progress
- `[x]` completed

## Task Board

| Status | ID | Task | Owner | Depends On | Files | Verification | Commit | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [ ] | P?-T1 | `<task summary>` | unassigned | - | `<paths>` | `<command>` | - | `<notes>` |
| [ ] | P?-T2 | `<task summary>` | unassigned | P?-T1 | `<paths>` | `<command>` | - | `<notes>` |

## Parallelization Rules

1. Only one owner per task while status is `[~]`.
2. Tasks with unmet dependencies stay `[ ]`.
3. Prefer tasks that touch different crates/files to avoid conflicts.
4. Update board status in the same commit that changes code when possible.
5. Keep tasks small enough to complete and verify independently.

## Phase Exit Criteria

1. Every task is `[x]`.
2. `cargo check --workspace` passes.
3. `cargo test --workspace` passes.
4. Work is merged into `main`.

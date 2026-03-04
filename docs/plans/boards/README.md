# Parallel Plan Boards

Use board-style phase plans when work can be split across multiple agents.

## Status Legend

- `[ ]` not started
- `[~]` in progress (claimed by one owner)
- `[x]` completed and merged to `main`

## Claim and Update Protocol

1. Pick one `[ ]` task whose dependencies are all `[x]`.
2. Change it to `[~]` and set `Owner`, `Started`, and short `Notes`.
3. Implement with tests, then commit.
4. Change it to `[x]`, fill `Commit`, and record verification evidence.
5. If blocked, keep `[~]` and write the blocker in `Notes`.

## Merge Protocol

1. When all tasks in the phase board are `[x]`, merge the phase into `main`.
2. Run verification on merged `main`.
3. Start the next phase from latest `main` using the template in `docs/plans/templates/`.

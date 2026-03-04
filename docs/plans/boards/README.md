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

## Parallel Dispatch Protocol

1. Dispatch in parallel only when task-owned files do not overlap.
2. Keep one owner per `[~]` row and include task ID in assignment prompts.
3. Require each owner to report task verification output before row completion.
4. After parallel work returns, verify on the controller branch before committing.

## Stage Command Quick Reference

The CLI supports stage inspection and execution with deterministic text or JSON output.

Examples:

1. List all stages in text mode (default):
   `cargo run -p cli -- stages`
2. List all stages in JSON mode:
   `cargo run -p cli -- stages --output json`
3. Run one stage in text mode (default):
   `cargo run -p cli -- run-stage normalize`
4. Run one stage in JSON mode:
   `cargo run -p cli -- run-stage normalize --output json`
5. Run full pipeline in text mode (default):
   `cargo run -p cli -- generate`
6. Run full pipeline in JSON mode:
   `cargo run -p cli -- generate --output json`

Unknown stage execution returns exit code `2` and an explicit error message.

Workflow note:

1. Use `stages` to inspect directories.
2. Use `run-stage` for targeted debugging of a single stage.
3. Use `generate` for full end-to-end artifact generation.

## Merge Protocol

1. When all tasks in the phase board are `[x]`, merge the phase into `main`.
2. Run verification on merged `main`.
3. Start the next phase from latest `main` using the template in `docs/plans/templates/`.

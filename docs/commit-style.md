# Commit Style Guide

## Format

Use Conventional Commits:

```text
<type>(<scope>): <summary>
```

Examples:

- `feat(layout): add zstack overlap heuristic`
- `fix(ui-spec): preserve spec_version on transform`
- `docs(workspace): add agent operating guide`

## Allowed Types

- `feat` - new behavior/capability
- `fix` - bug fix/regression fix
- `docs` - documentation only
- `refactor` - non-behavioral code restructuring
- `chore` - maintenance/tooling/setup
- `test` - tests or test tooling

## Allowed Scopes

- `cli`
- `figma-client`
- `normalizer`
- `layout`
- `ui-spec`
- `swiftui-ast`
- `swiftui-codegen`
- `assets`
- `review-report`
- `core`
- `workspace`
- `docs`
- `ci`

If a change spans several crates and no single owner is clear, use `workspace`.

## Summary Rules

1. Use imperative mood (`add`, `fix`, `update`), not past tense.
2. Keep it concise and specific.
3. Start lowercase after the colon.
4. Do not end with a period.

## Commit Scope Rules

1. Prefer one logical change per commit.
2. Separate refactors from behavior changes when possible.
3. Keep formatting-only edits separate from functional edits unless unavoidable.
4. Group related files that represent one coherent unit of work.

## Verification Before Commit

For code changes, run:

```bash
cargo check --workspace
cargo test --workspace
```

For docs-only commits, command runs are optional; include a short manual verification note.

## Commit Message Body (Recommended)

When useful, include:

1. Why the change is needed.
2. What changed at a high level.
3. Verification evidence.

Example:

```text
feat(core): add pipeline stage ordering contract

Define a canonical stage list used by CLI and orchestration checks.
This makes stage ordering explicit and testable.

Verification:
- cargo check --workspace
- cargo test --workspace
```

## Anti-Patterns

Avoid:

- `wip`
- `misc fixes`
- `update stuff`
- giant mixed commits with unrelated changes

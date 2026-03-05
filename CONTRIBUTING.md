# Contributing

Thanks for your interest in improving Forge.

## Development Setup

1. Install stable Rust (edition 2024 compatible).
2. Clone the repository and enter the project root.
3. Run the baseline verification:

```bash
cargo check --workspace
cargo test --workspace
bash scripts/verify_workspace.sh
```

## Workflow

1. Keep changes scoped to one clear goal.
2. Preserve deterministic stage behavior and explicit warnings.
3. Add or update tests when behavior changes.
4. Follow the commit format in [`docs/commit-style.md`](docs/commit-style.md).

## Pull Requests

1. Describe the user-visible behavior change.
2. Include verification evidence in the PR description.
3. Call out any follow-up work that you intentionally deferred.

## Code of Conduct

This project follows [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

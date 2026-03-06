# Release-Cached Instruction Loading Design

**Date:** 2026-03-06

## Goal

Replace per-file raw GitHub instruction fetching with tagged release snapshot caching under `~/.config/specloom`, while preserving deterministic local-first behavior for bundle preparation.

## Problem

`prepare-llm-bundle` currently resolves instruction docs in this order:

1. workspace-local project files
2. cached remote files
3. per-file GitHub raw URLs based on `v<version>` / `<version>`

This has two issues:

1. the raw URL format is brittle and currently incorrect for this repository
2. fetching files one-by-one creates unnecessary network coupling and partial-cache states

## Approved Direction

Use release snapshots instead of raw file URLs.

Lookup order becomes:

1. workspace-local instruction files
2. extracted release snapshot under `~/.config/specloom/release_cache/<tag>/`
3. download tagged release snapshot for the running version
4. if no matching release exists, fetch the latest release tag and download that snapshot
5. if no release snapshot can be resolved, return an explicit actionable error

## Cache Layout

Store extracted release snapshots under:

`~/.config/specloom/release_cache/<tag>/`

Each snapshot is expected to contain the repository-relative files used by bundle preparation:

1. `.codex/SKILLS.md`
2. `.codex/skills/...`
3. `docs/agent-playbook.md`
4. `docs/figma-ui-coder.md`

This keeps cached instruction sources versioned and reusable across runs.

## Download Strategy

For the current CLI version, try tags in this order:

1. `v<version>`
2. `<version>`

If neither tag resolves to a release snapshot, query the latest release metadata and use its tag.

The implementation should download one archive per resolved tag, extract only the required repository tree into the cache root, then load instruction files from the extracted snapshot.

## Error Handling

Failures should remain explicit:

1. local files missing + no cached snapshot + release lookup/download failure => actionable `FetchClient` error
2. missing required instruction file inside an extracted snapshot => actionable `MissingInputArtifact` error
3. cache path normalization must remain traversal-safe

## Testing

Add deterministic tests for:

1. loading instructions from a downloaded tagged release snapshot
2. falling back to the latest release when the running version tag is unavailable
3. preferring existing cached release snapshots without hitting the network
4. preferring workspace-local files over cached release content

## Docs Impact

Update user-facing docs to describe:

1. tagged release snapshot fallback instead of raw GitHub file fetching
2. cache location under `~/.config/specloom/release_cache/<tag>/`
3. latest-release fallback behavior when the current version is not yet released

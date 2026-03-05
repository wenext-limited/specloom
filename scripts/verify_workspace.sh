#!/usr/bin/env bash
set -euo pipefail

# Contract consistency checks across core stage crates.
cargo test -p forge-figma-normalizer defaults_include_explicit_versions
cargo test -p forge-figma-normalizer root_contract_fields_are_explicit
cargo test -p forge-layout-infer decision_contract_fields_are_explicit_and_ordered
cargo test -p forge-layout-infer decision_record_rejects_unknown_fields
cargo test -p forge-asset-pipeline manifest_contract_matches_next_stage_map
cargo test -p forge-asset-pipeline asset_entry_field_order_is_deterministic

# End-to-end CLI generate smoke and determinism checks.
cargo test -p forge-figma-pipeline --test e2e_generate

cargo fmt --all --check
cargo check --workspace
cargo test --workspace

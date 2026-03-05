#!/usr/bin/env bash
set -euo pipefail

# Contract consistency checks across core stage crates.
cargo test -p forge-figma-core defaults_include_explicit_versions
cargo test -p forge-figma-core root_contract_fields_are_explicit
cargo test -p forge-figma-core manifest_contract_matches_next_stage_map
cargo test -p forge-figma-core asset_entry_field_order_is_deterministic
cargo test -p forge-figma-core agent_context_round_trip_json

# End-to-end CLI generate smoke and determinism checks.
cargo test -p forge-figma-pipeline --test e2e_generate

cargo fmt --all --check
cargo check --workspace
cargo test --workspace

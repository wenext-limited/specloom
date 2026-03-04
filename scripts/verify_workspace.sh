#!/usr/bin/env bash
set -euo pipefail

# Contract consistency checks across core stage crates.
cargo test -p figma_normalizer defaults_include_explicit_versions
cargo test -p figma_normalizer root_contract_fields_are_explicit
cargo test -p layout_infer decision_contract_fields_are_explicit_and_ordered
cargo test -p layout_infer decision_record_rejects_unknown_fields
cargo test -p asset_pipeline manifest_contract_matches_next_stage_map
cargo test -p asset_pipeline asset_entry_field_order_is_deterministic
cargo test -p review_report summary_includes_zero_counts_for_all_categories_and_severities
cargo test -p review_report warning_contract_values_are_stable_snake_case

# End-to-end CLI generate smoke and determinism checks.
cargo test -p cli --test e2e_generate

cargo fmt --all --check
cargo check --workspace
cargo test --workspace

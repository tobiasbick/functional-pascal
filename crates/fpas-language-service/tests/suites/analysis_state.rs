//! Incremental analysis, checked diagnostics, source versions and snapshots.

#[path = "../support/mod.rs"]
mod support;

#[path = "../analysis.rs"]
mod analysis;

#[path = "../case_coverage.rs"]
mod case_coverage;

#[path = "../contracts.rs"]
mod contracts;

#[path = "../document_store.rs"]
mod document_store;

#[path = "../enum_values.rs"]
mod enum_values;

#[path = "../expression_type.rs"]
mod expression_type;

#[path = "../local_inference.rs"]
mod local_inference;

#[path = "../public_signatures.rs"]
mod public_signatures;

#[path = "../query_cache.rs"]
mod query_cache;

#[path = "../routine_visibility.rs"]
mod routine_visibility;

#[path = "../unicode_character_literals.rs"]
mod unicode_character_literals;

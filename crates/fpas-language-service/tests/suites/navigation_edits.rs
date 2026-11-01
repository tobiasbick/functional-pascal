//! Canonical symbol navigation, semantic identities, rename and source edits.

#[path = "../support/mod.rs"]
mod support;

#[path = "../checked_navigation.rs"]
mod checked_navigation;

#[path = "../import_aliases/main.rs"]
mod import_aliases;

#[path = "../navigation.rs"]
mod navigation;

#[path = "../navigation_resolution.rs"]
mod navigation_resolution;

#[path = "../refactoring.rs"]
mod refactoring;

#[path = "../refactoring_plans.rs"]
mod refactoring_plans;

#[path = "../references_rename.rs"]
mod references_rename;

#[path = "../rename_safety.rs"]
mod rename_safety;

#[path = "../semantic_tools.rs"]
mod semantic_tools;

#[path = "../workspace_navigation.rs"]
mod workspace_navigation;

#[path = "../source_query_find.rs"]
mod source_query_find;

#[path = "../refactoring_add_parameter.rs"]
mod refactoring_add_parameter;
#[path = "../source_query_calls.rs"]
mod source_query_calls;
#[path = "../source_query_inspect.rs"]
mod source_query_inspect;

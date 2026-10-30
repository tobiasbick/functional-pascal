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

#[path = "../references_rename.rs"]
mod references_rename;

#[path = "../rename_safety.rs"]
mod rename_safety;

#[path = "../semantic_tools.rs"]
mod semantic_tools;

#[path = "../workspace_navigation.rs"]
mod workspace_navigation;

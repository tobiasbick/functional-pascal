//! LSP backend and lifecycle handlers.

mod backend;
mod call_hierarchy;
mod errors;
mod initialization;
mod intellisense;
mod navigation;
mod public_signatures;
mod refactoring;
mod semantic_tools;
mod source_tools;
mod watched_files;

pub use backend::Backend;

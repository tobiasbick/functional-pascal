//! LSP backend and lifecycle handlers.

mod backend;
mod errors;
mod initialization;
mod intellisense;
mod navigation;
mod semantic_tools;
mod watched_files;

pub use backend::Backend;

//! LSP backend and lifecycle handlers.

mod backend;
mod initialization;
mod intellisense;
mod navigation;
mod semantic_tools;

pub use backend::Backend;

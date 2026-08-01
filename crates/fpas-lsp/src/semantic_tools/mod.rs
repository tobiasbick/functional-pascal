//! LSP conversion for semantic identifiers and compiler-backed quick fixes.

mod code_actions;
mod legend;
mod tokens;

pub(crate) use code_actions::{code_action, diagnostic_identity};
pub(crate) use legend::semantic_tokens_legend;
pub(crate) use tokens::semantic_tokens;

//! Protocol-neutral result of one seeded empty-storage initialization.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use crate::vm::debug::evaluation::DebugEvaluateResult;

/// Rendered summaries after one complete empty-storage descendant initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStorageInitializationResult {
    /// Visible root binding that received the complete seeded value.
    pub root: String,
    /// Canonical descendant target resolved against the detached seed.
    pub target: String,
    /// Bounded display of the committed complete root.
    pub root_value: String,
    /// Fresh retained summary of the selected descendant.
    pub value: DebugEvaluateResult,
}

//! Protocol-neutral textual debugger assignment targets.

use super::super::evaluation::{DebugEvaluateResult, DebugExpression};

/// A visible debugger binding followed by bounded stored-value selectors.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugAssignmentTarget {
    /// Visible local, parameter, capture, or global name.
    pub root: String,
    /// Stored field and evaluated index selectors in source order.
    pub selectors: Vec<DebugAssignmentSelector>,
}

/// One selector below a textual debugger assignment root.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugAssignmentSelector {
    /// Stored record field selected case-insensitively.
    Field(String),
    /// Array index or existing dictionary key expression.
    Index(DebugExpression),
}

/// Rendered result and operation metadata for one dictionary structure mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugDictionaryMutationResult {
    /// Fresh retained summary of the committed dictionary container.
    pub dictionary: DebugEvaluateResult,
    /// Bounded summary of the removed value, when an entry was removed.
    pub removed: Option<String>,
    /// Bounded summary of the replaced key, when a key was replaced.
    pub old_key: Option<String>,
    /// Bounded summary of the new key, when a key was replaced.
    pub new_key: Option<String>,
}

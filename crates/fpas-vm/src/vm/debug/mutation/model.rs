//! Protocol-neutral textual debugger assignment targets.

use super::super::evaluation::DebugExpression;

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

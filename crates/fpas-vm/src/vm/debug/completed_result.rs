//! Protocol-neutral replacement result for one completed retained task.

/// Rendered result of replacing one unconsumed retained task result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugTaskResultReplacement {
    /// Runtime identity of the completed retained task.
    pub task_id: u64,
    /// Bounded FPAS value summary of the replacement.
    pub value: String,
    /// Runtime or source type name of the replacement.
    pub type_name: String,
    /// Stop-local reference for aggregate expansion, or zero for a leaf.
    pub variables_reference: u64,
    /// Number of named children.
    pub named_variables: usize,
    /// Number of indexed children.
    pub indexed_variables: usize,
}

//! Explicit code-generation policy; semantic analysis always checks the full source.

/// Code-generation options shared by standalone programs and separately compiled units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileOptions {
    /// Generate and evaluate `requires` and `ensures` clauses.
    pub contracts: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self { contracts: true }
    }
}

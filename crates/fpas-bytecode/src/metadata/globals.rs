//! Dense global-slot declarations.

use crate::StringId;

/// Metadata for one executable-wide global slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalInfo {
    /// Canonical diagnostic name in the string table.
    pub name: StringId,
    /// Whether bytecode may store a new value after initialization.
    pub mutable: bool,
}

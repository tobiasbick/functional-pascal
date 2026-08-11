//! Enum type layouts and executable-wide variant slots.

use crate::{DebugTypeId, EnumTypeId, StringId};

/// Metadata for one enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLayout {
    /// Canonical enum type name in the string table.
    pub name: StringId,
}

/// Metadata for one executable-wide enum variant identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    /// Enum type that owns this variant.
    pub owner: EnumTypeId,
    /// Canonical variant name in the string table.
    pub name: StringId,
    /// Canonical associated-field names in positional order.
    pub fields: Vec<StringId>,
    /// Associated field types in positional order.
    pub field_types: Vec<DebugTypeId>,
}

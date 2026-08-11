//! Record layouts and field slots.

use crate::{DebugTypeId, StringId};

/// Metadata for a record-local positional field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordField {
    /// Canonical field name in the string table.
    pub name: StringId,
    /// Machine-readable stored field type.
    pub ty: DebugTypeId,
}

/// Ordered field layout for one record type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// Canonical record type name in the string table.
    pub name: StringId,
    /// Fields in numeric slot order.
    pub fields: Vec<RecordField>,
    /// Readable source properties and exact canonical getter routines.
    pub properties: Vec<RecordProperty>,
}

/// Property-to-getter mapping used by exact debugger member binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordProperty {
    /// Public source property name.
    pub name: StringId,
    /// Canonical qualified getter routine name.
    pub getter: StringId,
}

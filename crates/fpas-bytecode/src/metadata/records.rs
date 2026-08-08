//! Record layouts and field slots.

use crate::StringId;

/// Metadata for a record-local positional field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordField {
    /// Canonical field name in the string table.
    pub name: StringId,
}

/// Ordered field layout for one record type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// Canonical record type name in the string table.
    pub name: StringId,
    /// Fields in numeric slot order.
    pub fields: Vec<RecordField>,
}

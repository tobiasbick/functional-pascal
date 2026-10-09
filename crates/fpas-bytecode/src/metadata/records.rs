//! Record layouts and field slots.

use crate::{DebugTypeId, SourceId, StringId};

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
    /// Instance methods and exact canonical routines.
    pub methods: Vec<RecordMethod>,
    /// Typed construction metadata; see `docs/pascal/language/types/records.md`.
    pub construction: Option<RecordConstructionInfo>,
}

/// Source-visible nominal type names, construction access, and field defaults.
/// See `docs/pascal/language/types/records.md` and `docs/pascal/tools/debugger.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordConstructionInfo {
    /// Declaring unit, absent for program-owned records.
    pub owner_unit: Option<StringId>,
    /// Whether construction requires the declaring unit's scope.
    pub requires_owner: bool,
    /// Visible names scoped to the source where they were declared or imported.
    pub aliases: Vec<RecordTypeAlias>,
    /// Zero-argument default routine names in stored field order.
    pub defaults: Vec<Option<StringId>>,
}

/// One source-visible name for the same nominal record type.
/// See `docs/pascal/language/types/records.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordTypeAlias {
    /// Source containing the type declaration, type alias, or import.
    pub source: SourceId,
    /// Source-visible type name, including a uses alias when applicable.
    pub name: StringId,
    /// Unit or program owning this source scope.
    pub unit: StringId,
}

/// Method-to-routine mapping used by exact debugger bound-receiver construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordMethod {
    /// Source method member name.
    pub name: StringId,
    /// Canonical qualified executable routine name.
    pub routine: StringId,
}

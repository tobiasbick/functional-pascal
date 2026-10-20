//! Essential native-error metadata, independent of optional debugger types.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

mod identity;

pub use identity::{ErrorInstanceKey, IdentityParameter, NominalKind, TypeIdentity};

use crate::{EnumTypeId, StringId};

/// Checked location and concrete nominal identity of one original cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCause {
    /// Associated-field position in the owning variant.
    pub field: u16,
    /// Executable-local concrete error instance expected in this field.
    pub family: EnumTypeId,
}

/// Error-specific description for one variant of an error family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorVariant {
    /// Concrete semantic field types, independent of optional debugger metadata.
    pub field_types: Vec<TypeIdentity>,
    /// Optional nonempty literal label in the executable string table.
    pub message: Option<StringId>,
    /// Original typed cause, absent for leaf variants.
    pub cause: Option<ErrorCause>,
}

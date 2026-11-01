//! Compiler-selected source bindings retained for navigation and call inventories.

use crate::Ty;

/// The declaration category matters independently of its (possibly callable) value type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBindingKind {
    /// Named source function or procedure, including imported routines.
    Routine,
    /// Constant, variable, parameter or loop binding; callable values remain dynamic.
    Value,
    /// Named type used by constructors, conversions or static member lookup.
    Type,
    /// Enum member or associated-data constructor.
    EnumConstructor,
    /// Polymorphic compiler-owned standard operation.
    Native,
}

/// A resolved designator prefix at its exact source site, without name-based re-resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceBinding {
    /// Number of identifier parts consumed before any value/member/index suffix.
    pub parts: usize,
    /// Local declaration identity as `(source_id, byte_offset)`, when source-owned here.
    pub declaration: Option<(u32, usize)>,
    /// Compiler-canonical import identity, or local spelling when a local span is present.
    pub name: String,
    /// Declaring category, independent of inferred value type.
    pub kind: SourceBindingKind,
    /// Checked type of the selected prefix.
    pub ty: Ty,
}

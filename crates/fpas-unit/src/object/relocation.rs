//! Explicit relocation records for packed register instructions.

use crate::object::SymbolReference;

/// One function-local instruction operand rewritten by the linker.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Relocation {
    /// Object-local function index.
    pub function: u32,
    /// Function-local instruction index.
    pub instruction: u32,
    /// Operand category and symbolic target.
    pub kind: RelocationKind,
}

/// Relocatable register-bytecode operand.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelocationKind {
    /// Object-local constant table index.
    Constant(u32),
    /// Callable reference used by a direct call or closure.
    Function(SymbolReference),
    /// Dense global slot reference.
    Global(SymbolReference),
    /// Record layout reference.
    Record(SymbolReference),
    /// Record-local field slot, validated against linked layouts.
    RecordField(u16),
    /// Enum variant in an object-local or imported enum layout.
    EnumVariant {
        /// Owning enum symbol.
        enumeration: SymbolReference,
        /// Canonical variant name.
        variant: String,
    },
    /// Enum associated-field slot, validated against linked layouts.
    EnumField(u16),
    /// Function-local branch target.
    CodeAddress(u32),
}

//! Runtime-independent constants, globals, layouts, and source metadata.

use crate::object::SymbolReference;

/// Persistent object constant using semantic bit identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ObjectConstant {
    /// Signed integer.
    Integer(i64),
    /// Exact IEEE-754 bit pattern.
    Real(u64),
    /// Boolean value.
    Boolean(bool),
    /// UTF-8 string value.
    String(String),
    /// Procedure result value.
    Unit,
    /// Non-capturing function value.
    Function {
        /// Symbolic or object-local callable reference.
        function: SymbolReference,
        /// Whether invocation is task-bound.
        task_bound: bool,
    },
}

/// Object-local global slot declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectGlobal {
    /// Canonical diagnostic name.
    pub name: String,
    /// Whether stores are allowed after initialization.
    pub mutable: bool,
}

/// Ordered record layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectRecordLayout {
    /// Canonical type name.
    pub name: String,
    /// Canonical field names in declaration order.
    pub fields: Vec<String>,
}

/// One enum variant and its associated fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectEnumVariant {
    /// Canonical variant name.
    pub name: String,
    /// Canonical associated-field names in declaration order.
    pub fields: Vec<String>,
}

/// Ordered enum layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectEnumLayout {
    /// Canonical type name.
    pub name: String,
    /// Variants in declaration order.
    pub variants: Vec<ObjectEnumVariant>,
}

/// Sparse source location using an object-local source-path ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectSourceRun {
    /// Function-local first instruction using this location.
    pub instruction_start: u32,
    /// Object-local source path index.
    pub source: u32,
    /// One-based line.
    pub line: u32,
    /// One-based column.
    pub column: u32,
}

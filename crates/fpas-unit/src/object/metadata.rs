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
    /// Readable properties and canonical getter routine names.
    pub properties: Vec<ObjectRecordProperty>,
}

/// Relocatable property-to-getter mapping.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectRecordProperty {
    /// Public source property name.
    pub name: String,
    /// Canonical qualified getter routine name.
    pub getter: String,
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

/// Complete relocatable debugger metadata for one object function.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectFunctionDebugInfo {
    /// Dense lexical scope tree.
    pub scopes: Vec<ObjectDebugScope>,
    /// Source-visible register bindings.
    pub bindings: Vec<ObjectDebugBinding>,
    /// Ordered function-local sequence points.
    pub sequence_points: Vec<ObjectSequencePoint>,
}

/// One function-local lexical scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectDebugScope {
    /// Dense scope identifier.
    pub id: u32,
    /// Parent scope, absent only for the root.
    pub parent: Option<u32>,
}

/// Source-level role of an object debug binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectDebugBindingKind {
    /// Explicit routine parameter.
    Parameter,
    /// Lexically declared local variable.
    Local,
    /// Captured value.
    Capture,
}

/// Relocatable source location using an object-local source ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectDebugLocation {
    /// Object-local source path index.
    pub source: u32,
    /// One-based line.
    pub line: u32,
    /// One-based column.
    pub column: u32,
}

/// Source-visible object binding backed by a function-local register.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectDebugBinding {
    /// Source name.
    pub name: String,
    /// Portable display type.
    pub type_name: String,
    /// Function-local register.
    pub register: u16,
    /// Source-level role.
    pub kind: ObjectDebugBindingKind,
    /// Whether source semantics permit reassignment.
    pub mutable: bool,
    /// Lexical scope identifier.
    pub scope: u32,
    /// Declaration location when available.
    pub declaration: Option<ObjectDebugLocation>,
    /// Whether compiler-generated storage is hidden from normal scopes.
    pub hidden: bool,
    /// Whether the register stores a mutable capture cell.
    pub cell_backed: bool,
}

/// A function-local debugger sequence point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectSequencePoint {
    /// Function-local instruction index.
    pub instruction_start: u32,
    /// Source location represented by the point.
    pub location: ObjectDebugLocation,
    /// Innermost active lexical scope.
    pub scope: u32,
}

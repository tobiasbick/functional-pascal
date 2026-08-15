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
    /// Object-local portable debugger type identifier.
    pub ty: u32,
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
    /// Object-local debugger types for fields in declaration order.
    pub field_types: Vec<u32>,
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
    /// Object-local debugger types for associated fields.
    pub field_types: Vec<u32>,
}

/// Portable object-local debugger type graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ObjectDebugType {
    /// Procedure result type.
    Unit,
    /// Boolean value.
    Boolean,
    /// Signed integer value.
    Integer,
    /// IEEE-754 real value.
    Real,
    /// UTF-8 string value.
    String,
    /// Dynamically checked value.
    Dynamic,
    /// Ordered array element type.
    Array(u32),
    /// Dictionary key and value types.
    Dictionary {
        /// Key type.
        key: u32,
        /// Value type.
        value: u32,
    },
    /// Result success and error types.
    Result {
        /// Success type.
        ok: u32,
        /// Error type.
        error: u32,
    },
    /// Optional inner type.
    Option(u32),
    /// First-class function signature.
    Function {
        /// Parameter types.
        parameters: Vec<u32>,
        /// Result type.
        result: u32,
    },
    /// Record layout by canonical name.
    Record(String),
    /// Enum layout by canonical name.
    Enum(String),
    /// Mutable cell inner type.
    Cell(u32),
    /// Task result type.
    Task(u32),
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
    /// Object-local portable result type, absent when metadata was not retained.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[serde(default)]
    pub result_type: Option<u32>,
    /// Object-local lexical owner function index; absent when the function has no captures.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[serde(default)]
    pub lexical_owner: Option<u32>,
    /// Capture identity in runtime closure ABI order, using object-local binding indexes.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[serde(default)]
    pub capture_sources: Vec<ObjectCaptureSource>,
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
    /// Object-local portable debugger type identifier.
    pub ty: u32,
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

/// Object-local capture identity for one nested-function capture.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectCaptureSource {
    /// Object-local owner binding index.
    pub binding: u32,
    /// Object-local portable debugger type identifier.
    pub ty: u32,
    /// Representation mandated by semantic capture analysis.
    pub kind: ObjectCaptureKind,
}

/// Capture representation stored in relocatable object metadata.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectCaptureKind {
    /// The closure captures an immutable value.
    Value,
    /// The closure captures a mutable cell.
    Cell,
    /// The closure reuses an enclosing mutable cell.
    EnclosingCell,
}

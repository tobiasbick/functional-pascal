//! Ordered program-wide IR tables and compact lowered types.

use crate::{
    EnumLayoutId, FieldId, Function, FunctionId, GlobalId, IntrinsicId, RecordLayoutId, TypeId,
    VariantId,
};

/// A complete deterministic typed IR program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Types in explicit deterministic order.
    pub types: Vec<TypeDefinition>,
    /// Global declarations in explicit deterministic order.
    pub globals: Vec<Global>,
    /// Record layouts in explicit deterministic order.
    pub record_layouts: Vec<RecordLayout>,
    /// Enum layouts in explicit deterministic order.
    pub enum_layouts: Vec<EnumLayout>,
    /// Intrinsic signatures in explicit deterministic order.
    pub intrinsics: Vec<IntrinsicSignature>,
    /// Distinct type names visible in the compiling source, for debugger conversions.
    pub distinct_types: Vec<DistinctTypeName>,
    /// Functions in explicit deterministic order.
    pub functions: Vec<Function>,
    /// The function selected as the root entry point.
    pub entry: FunctionId,
}

impl Program {
    /// Returns a type definition by its typed identifier.
    #[must_use]
    pub fn ty(&self, id: TypeId) -> Option<&TypeDefinition> {
        self.types.iter().find(|definition| definition.id == id)
    }

    /// Returns a function by its typed identifier.
    #[must_use]
    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.iter().find(|function| function.id == id)
    }

    /// Returns a global declaration by its typed identifier.
    #[must_use]
    pub fn global(&self, id: GlobalId) -> Option<&Global> {
        self.globals.iter().find(|global| global.id == id)
    }

    /// Returns a record layout by its typed identifier.
    #[must_use]
    pub fn record_layout(&self, id: RecordLayoutId) -> Option<&RecordLayout> {
        self.record_layouts.iter().find(|layout| layout.id == id)
    }

    /// Returns an enum layout by its typed identifier.
    #[must_use]
    pub fn enum_layout(&self, id: EnumLayoutId) -> Option<&EnumLayout> {
        self.enum_layouts.iter().find(|layout| layout.id == id)
    }

    /// Returns an intrinsic signature by its typed identifier.
    #[must_use]
    pub fn intrinsic(&self, id: IntrinsicId) -> Option<&IntrinsicSignature> {
        self.intrinsics.iter().find(|intrinsic| intrinsic.id == id)
    }
}

/// A compact lowered type used by IR validation and later code generation.
///
/// This intentionally represents only distinctions that survive semantic analysis into executable
/// code; it does not duplicate semantic names, visibility, or source-level generic metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    /// The procedure result type.
    Unit,
    /// A boolean value.
    Boolean,
    /// A signed integer value.
    Integer,
    /// An IEEE-754 real value.
    Real,
    /// A UTF-8 string value.
    String,
    /// A generic-erased value whose operation remains dynamically checked.
    Dynamic,
    /// Ordered value-semantic array.
    Array(TypeId),
    /// Opaque bounded channel with a statically known element type.
    Channel(TypeId),
    /// Ordered value-semantic dictionary.
    Dictionary {
        /// Key type.
        key: TypeId,
        /// Stored value type.
        value: TypeId,
    },
    /// Result value with success and error payload types.
    Result {
        /// Success payload type.
        ok: TypeId,
        /// Error payload type.
        error: TypeId,
    },
    /// Optional value.
    Option(TypeId),
    /// A function value with ordered parameter and result types.
    Function {
        /// Ordered parameter types.
        parameters: Vec<TypeId>,
        /// Result type.
        result: TypeId,
    },
    /// A record value with a validated layout.
    Record(RecordLayoutId),
    /// An enum value with a validated layout.
    Enum(EnumLayoutId),
    /// A mutable capture cell containing a value of this type.
    Cell(TypeId),
    /// A task handle whose result type is known to the compiler.
    Task(TypeId),
    /// A reference to a caller variable, record field, or array element of this type.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    Reference(TypeId),
}

/// An identified lowered type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    /// Stable type identifier.
    pub id: TypeId,
    /// Lowered type category.
    pub kind: IrType,
}

/// A global declaration with its resolved lowered type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global {
    /// Stable global identifier.
    pub id: GlobalId,
    /// Canonical source-level name used for diagnostics and display.
    pub name: String,
    /// Type of values stored in the global.
    pub ty: TypeId,
    /// Whether stores after initialization are permitted.
    pub mutable: bool,
    /// Exact source-declaration store, when this global is source-defined.
    pub initializer: Option<GlobalInitializer>,
}

/// Exact IR identity of one global source initializer store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalInitializer {
    /// Function containing the store.
    pub function: FunctionId,
    /// Function-local IR instruction identity.
    pub location: crate::DebugInstructionLocation,
}

/// A record layout known to the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// Stable record-layout identifier.
    pub id: RecordLayoutId,
    /// Canonical qualified record name.
    pub name: String,
    /// Field declarations in declaration order.
    pub fields: Vec<RecordField>,
    /// Instance methods and their exact compiler-resolved routine names.
    pub methods: Vec<RecordMethod>,
    /// Typed debugger construction; see `docs/pascal/language/types/records.md`.
    pub construction: Option<RecordConstructionInfo>,
}

/// One distinct type name visible in the compiling source and its scalar underlying type.
/// See `docs/pascal/language/types/distinct-types.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistinctTypeName {
    /// Source-visible type name, including a uses alias when applicable.
    pub name: String,
    /// Scalar underlying type.
    pub underlying: TypeId,
}

/// Declaration-bound defaults and visible names retained for debugger construction.
/// See `docs/pascal/language/types/records.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordConstructionInfo {
    /// Declaring unit, absent for program-owned records.
    pub owner_unit: Option<String>,
    /// Whether any field restricts construction to the declaring unit.
    pub requires_owner: bool,
    /// Visible type names and aliases in the compiling source.
    pub aliases: Vec<String>,
    /// Compiling source's unit or program name.
    pub scope_unit: String,
    /// Zero-argument field-default routines in field declaration order.
    pub defaults: Vec<Option<String>>,
}

/// Instance-method mapping retained for debugger-side bound receiver values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordMethod {
    /// Public source member spelling.
    pub name: String,
    /// Canonical qualified routine name resolved by semantic analysis.
    pub routine: String,
}

/// A field inside a record layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    /// Stable field identifier within the layout.
    pub id: FieldId,
    /// Canonical declared field name.
    pub name: String,
    /// Type stored in the field.
    pub ty: TypeId,
}

/// An enum layout known to the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLayout {
    /// Stable enum-layout identifier.
    pub id: EnumLayoutId,
    /// Canonical qualified enum name.
    pub name: String,
    /// Variant declarations in declaration order.
    pub variants: Vec<EnumVariant>,
}

/// A variant inside an enum layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    /// Stable variant identifier within the layout.
    pub id: VariantId,
    /// Canonical declared variant name.
    pub name: String,
    /// Canonical associated-field names in declaration order.
    pub field_names: Vec<String>,
    /// Associated-value types in declaration order.
    pub fields: Vec<TypeId>,
}

/// A statically known intrinsic call signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrinsicSignature {
    /// Stable intrinsic identifier.
    pub id: IntrinsicId,
    /// Ordered parameter types.
    pub parameters: Vec<TypeId>,
    /// Whether the final parameter type repeats for additional arguments.
    pub variadic: bool,
    /// Result type.
    pub result: TypeId,
}

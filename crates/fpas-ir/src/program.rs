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
    /// Type of values stored in the global.
    pub ty: TypeId,
}

/// A record layout known to the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// Stable record-layout identifier.
    pub id: RecordLayoutId,
    /// Field declarations in declaration order.
    pub fields: Vec<RecordField>,
}

/// A field inside a record layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    /// Stable field identifier within the layout.
    pub id: FieldId,
    /// Type stored in the field.
    pub ty: TypeId,
}

/// An enum layout known to the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumLayout {
    /// Stable enum-layout identifier.
    pub id: EnumLayoutId,
    /// Variant declarations in declaration order.
    pub variants: Vec<EnumVariant>,
}

/// A variant inside an enum layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    /// Stable variant identifier within the layout.
    pub id: VariantId,
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
    /// Result type.
    pub result: TypeId,
}

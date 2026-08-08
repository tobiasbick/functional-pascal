//! Typed three-address IR operations.

use crate::{
    EnumLayoutId, FieldId, FunctionId, GlobalId, IntrinsicId, LocalId, RecordLayoutId, SourceSpan,
    ValueDefinition, ValueId, VariantId,
};

/// An instruction with an optional typed result definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// Source span for a semantic operation, or `None` for compiler-synthesized work.
    pub source: Option<SourceSpan>,
    /// Result definition for value-producing operations.
    pub result: Option<ValueDefinition>,
    /// Typed operation evaluated by this instruction.
    pub operation: Operation,
}

/// A constant representable in target-independent IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// The Unit value.
    Unit,
    /// A boolean value.
    Boolean(bool),
    /// A signed integer value.
    Integer(i64),
    /// An IEEE-754 real value.
    Real(f64),
    /// A UTF-8 string value.
    String(String),
}

/// A typed binary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    /// Integer addition.
    AddInteger,
    /// Integer subtraction.
    SubtractInteger,
    /// Integer multiplication.
    MultiplyInteger,
    /// Integer division.
    DivideInteger,
    /// Integer remainder.
    RemainderInteger,
    /// Real addition.
    AddReal,
    /// Real subtraction.
    SubtractReal,
    /// Real multiplication.
    MultiplyReal,
    /// Real division.
    DivideReal,
    /// A dynamically checked generic numeric addition.
    AddDynamic,
    /// A dynamically checked generic numeric subtraction.
    SubtractDynamic,
    /// A dynamically checked generic numeric multiplication.
    MultiplyDynamic,
    /// A dynamically checked generic numeric division.
    DivideDynamic,
    /// Equality comparison of like-typed values.
    Equal,
    /// Inequality comparison of like-typed values.
    NotEqual,
    /// Integer less-than comparison.
    LessThanInteger,
    /// Integer greater-than comparison.
    GreaterThanInteger,
    /// Integer less-than-or-equal comparison.
    LessEqualInteger,
    /// Integer greater-than-or-equal comparison.
    GreaterEqualInteger,
    /// Real less-than comparison.
    LessThanReal,
    /// Real greater-than comparison.
    GreaterThanReal,
    /// Real less-than-or-equal comparison.
    LessEqualReal,
    /// Real greater-than-or-equal comparison.
    GreaterEqualReal,
    /// Dynamically checked less-than comparison.
    LessThanDynamic,
    /// Dynamically checked greater-than comparison.
    GreaterThanDynamic,
    /// Dynamically checked less-than-or-equal comparison.
    LessEqualDynamic,
    /// Dynamically checked greater-than-or-equal comparison.
    GreaterEqualDynamic,
    /// Boolean conjunction.
    AndBoolean,
    /// Boolean disjunction.
    OrBoolean,
    /// UTF-8 string concatenation.
    ConcatString,
    /// Integer left shift.
    ShiftLeftInteger,
    /// Integer right shift.
    ShiftRightInteger,
    /// Integer bitwise conjunction.
    BitAndInteger,
    /// Integer bitwise disjunction.
    BitOrInteger,
    /// Integer bitwise exclusive disjunction.
    BitXorInteger,
}

/// A typed unary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    /// Checked integer negation.
    NegateInteger,
    /// IEEE-754 real negation.
    NegateReal,
    /// Dynamically checked generic numeric negation.
    NegateDynamic,
    /// Boolean negation.
    NotBoolean,
    /// Convert an integer value to a real value.
    IntegerToReal,
}

/// A typed, target-independent operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Produces a scalar constant.
    Const(Constant),
    /// Reads an explicit local into a value.
    ReadLocal(LocalId),
    /// Writes a value to an explicit local.
    WriteLocal {
        /// Value written into the local.
        value: ValueId,
        /// Target local.
        local: LocalId,
    },
    /// Evaluates a typed binary operation.
    Binary {
        /// Chosen typed operation.
        operation: BinaryOperation,
        /// Left operand.
        left: ValueId,
        /// Right operand.
        right: ValueId,
    },
    /// Evaluates a typed unary operation.
    Unary {
        /// Chosen typed operation.
        operation: UnaryOperation,
        /// Source operand.
        operand: ValueId,
    },
    /// Calls a semantically resolved function directly.
    CallDirect {
        /// Target function.
        function: FunctionId,
        /// Arguments in evaluation order.
        arguments: Vec<ValueId>,
    },
    /// Calls a first-class function value.
    CallValue {
        /// Callee function value.
        callee: ValueId,
        /// Arguments in evaluation order.
        arguments: Vec<ValueId>,
    },
    /// Reads a dense global slot.
    LoadGlobal(GlobalId),
    /// Writes a dense global slot.
    StoreGlobal {
        /// Target global.
        global: GlobalId,
        /// Value written into the global.
        value: ValueId,
    },
    /// Constructs an array from elements in source order.
    MakeArray(Vec<ValueId>),
    /// Appends one value directly to a local array while preserving copy-on-write value semantics.
    ArrayPush {
        /// Local array updated by the operation.
        local: LocalId,
        /// Value appended to the array.
        value: ValueId,
    },
    /// Constructs an insertion-ordered dictionary.
    MakeDictionary(Vec<(ValueId, ValueId)>),
    /// Reads an array element or dictionary value.
    IndexGet {
        /// Indexed collection.
        collection: ValueId,
        /// Array index or dictionary key.
        index: ValueId,
    },
    /// Produces a copy-on-write aggregate with one indexed value replaced.
    IndexSet {
        /// Original collection.
        collection: ValueId,
        /// Array index or dictionary key.
        index: ValueId,
        /// Replacement value.
        value: ValueId,
    },
    /// Tests array membership or dictionary-key membership.
    Contains {
        /// Searched element or key.
        value: ValueId,
        /// Array or dictionary.
        collection: ValueId,
    },
    /// Constructs a record from layout-ordered fields.
    MakeRecord {
        /// Record layout.
        layout: RecordLayoutId,
        /// Field values in declaration order.
        fields: Vec<ValueId>,
    },
    /// Reads one field from a record layout.
    LoadField {
        /// Record value.
        record: ValueId,
        /// Expected record layout.
        layout: RecordLayoutId,
        /// Field inside the layout.
        field: FieldId,
    },
    /// Stores one field through a record layout.
    StoreField {
        /// Record value.
        record: ValueId,
        /// Expected record layout.
        layout: RecordLayoutId,
        /// Field inside the layout.
        field: FieldId,
        /// Replacement field value.
        value: ValueId,
    },
    /// Produces a record with positional field overrides.
    UpdateRecord {
        /// Original record.
        record: ValueId,
        /// Expected layout.
        layout: RecordLayoutId,
        /// Numeric field/value pairs in source evaluation order.
        fields: Vec<(FieldId, ValueId)>,
    },
    /// Wraps a success payload.
    MakeOk(ValueId),
    /// Wraps an error payload.
    MakeError(ValueId),
    /// Wraps an optional payload.
    MakeSome(ValueId),
    /// Constructs an empty option.
    MakeNone,
    /// Tests whether a Result is successful.
    IsResultOk(ValueId),
    /// Tests whether an Option contains a value.
    IsOptionSome(ValueId),
    /// Extracts a success payload.
    UnwrapOk(ValueId),
    /// Extracts an error payload.
    UnwrapError(ValueId),
    /// Extracts an optional payload.
    UnwrapSome(ValueId),
    /// Constructs an enum variant with ordered associated values.
    MakeEnum {
        /// Enum layout.
        layout: EnumLayoutId,
        /// Variant inside the layout.
        variant: VariantId,
        /// Associated values in declaration order.
        fields: Vec<ValueId>,
    },
    /// Tests whether an enum value has one variant.
    TestVariant {
        /// Enum value.
        value: ValueId,
        /// Expected enum layout.
        layout: EnumLayoutId,
        /// Variant inside the layout.
        variant: VariantId,
    },
    /// Reads one associated enum field by positional slot.
    LoadEnumField {
        /// Enum value.
        value: ValueId,
        /// Expected enum layout.
        layout: EnumLayoutId,
        /// Expected active variant.
        variant: VariantId,
        /// Associated field slot.
        field: FieldId,
    },
    /// Invokes a registered intrinsic.
    Intrinsic {
        /// Intrinsic signature identifier.
        intrinsic: IntrinsicId,
        /// Arguments in evaluation order.
        arguments: Vec<ValueId>,
    },
    /// Creates a closure using semantic capture order.
    MakeClosure {
        /// Target function.
        function: FunctionId,
        /// Captured values in semantic capture order.
        captures: Vec<ValueId>,
    },
    /// Wraps a value in a shared mutable capture cell.
    MakeCell(ValueId),
    /// Reads the value inside a mutable capture cell.
    CellRead(ValueId),
    /// Writes a value into a mutable capture cell.
    CellWrite {
        /// Target cell value.
        cell: ValueId,
        /// Replacement cell content.
        value: ValueId,
    },
    /// Spawns a task from a function value.
    SpawnTask {
        /// Callee function value.
        callee: ValueId,
        /// Task arguments in evaluation order.
        arguments: Vec<ValueId>,
    },
    /// Spawns a detached task from a function value.
    SpawnDetachedTask {
        /// Callee function value.
        callee: ValueId,
        /// Task arguments in evaluation order.
        arguments: Vec<ValueId>,
    },
    /// Cooperatively yields the current task.
    Yield,
}

impl Operation {
    /// Returns whether this operation must define a result value.
    #[must_use]
    pub const fn produces_value(&self) -> bool {
        matches!(
            self,
            Self::Const(_)
                | Self::ReadLocal(_)
                | Self::Binary { .. }
                | Self::Unary { .. }
                | Self::CallDirect { .. }
                | Self::CallValue { .. }
                | Self::LoadGlobal(_)
                | Self::MakeArray(_)
                | Self::ArrayPush { .. }
                | Self::MakeDictionary(_)
                | Self::IndexGet { .. }
                | Self::IndexSet { .. }
                | Self::Contains { .. }
                | Self::MakeRecord { .. }
                | Self::LoadField { .. }
                | Self::UpdateRecord { .. }
                | Self::MakeOk(_)
                | Self::MakeError(_)
                | Self::MakeSome(_)
                | Self::MakeNone
                | Self::IsResultOk(_)
                | Self::IsOptionSome(_)
                | Self::UnwrapOk(_)
                | Self::UnwrapError(_)
                | Self::UnwrapSome(_)
                | Self::MakeEnum { .. }
                | Self::TestVariant { .. }
                | Self::LoadEnumField { .. }
                | Self::Intrinsic { .. }
                | Self::MakeClosure { .. }
                | Self::MakeCell(_)
                | Self::CellRead(_)
                | Self::SpawnTask { .. }
        )
    }
}

/// Returns the required operand and result categories for a binary operation.
#[must_use]
pub const fn binary_categories(operation: BinaryOperation) -> (TypeCategory, TypeCategory) {
    match operation {
        BinaryOperation::AddInteger
        | BinaryOperation::SubtractInteger
        | BinaryOperation::MultiplyInteger
        | BinaryOperation::DivideInteger
        | BinaryOperation::RemainderInteger
        | BinaryOperation::ShiftLeftInteger
        | BinaryOperation::ShiftRightInteger
        | BinaryOperation::BitAndInteger
        | BinaryOperation::BitOrInteger
        | BinaryOperation::BitXorInteger => (TypeCategory::Integer, TypeCategory::Integer),
        BinaryOperation::AddReal
        | BinaryOperation::SubtractReal
        | BinaryOperation::MultiplyReal
        | BinaryOperation::DivideReal => (TypeCategory::Real, TypeCategory::Real),
        BinaryOperation::AddDynamic
        | BinaryOperation::SubtractDynamic
        | BinaryOperation::MultiplyDynamic
        | BinaryOperation::DivideDynamic => (TypeCategory::Dynamic, TypeCategory::Dynamic),
        BinaryOperation::Equal | BinaryOperation::NotEqual => {
            (TypeCategory::Same, TypeCategory::Boolean)
        }
        BinaryOperation::LessThanInteger
        | BinaryOperation::GreaterThanInteger
        | BinaryOperation::LessEqualInteger
        | BinaryOperation::GreaterEqualInteger => (TypeCategory::Integer, TypeCategory::Boolean),
        BinaryOperation::LessThanReal
        | BinaryOperation::GreaterThanReal
        | BinaryOperation::LessEqualReal
        | BinaryOperation::GreaterEqualReal => (TypeCategory::Real, TypeCategory::Boolean),
        BinaryOperation::LessThanDynamic
        | BinaryOperation::GreaterThanDynamic
        | BinaryOperation::LessEqualDynamic
        | BinaryOperation::GreaterEqualDynamic => (TypeCategory::Comparable, TypeCategory::Boolean),
        BinaryOperation::AndBoolean | BinaryOperation::OrBoolean => {
            (TypeCategory::Boolean, TypeCategory::Boolean)
        }
        BinaryOperation::ConcatString => (TypeCategory::String, TypeCategory::String),
    }
}

/// Returns the required operand and result categories for a unary operation.
#[must_use]
pub const fn unary_categories(operation: UnaryOperation) -> (TypeCategory, TypeCategory) {
    match operation {
        UnaryOperation::NegateInteger => (TypeCategory::Integer, TypeCategory::Integer),
        UnaryOperation::NegateReal => (TypeCategory::Real, TypeCategory::Real),
        UnaryOperation::NegateDynamic => (TypeCategory::Dynamic, TypeCategory::Dynamic),
        UnaryOperation::NotBoolean => (TypeCategory::Boolean, TypeCategory::Boolean),
        UnaryOperation::IntegerToReal => (TypeCategory::Integer, TypeCategory::Real),
    }
}

/// A compact type category used when validating typed operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCategory {
    /// Requires matching operand types.
    Same,
    /// Requires the lowered Unit type.
    Unit,
    /// Requires the lowered boolean type.
    Boolean,
    /// Requires the lowered integer type.
    Integer,
    /// Requires the lowered real type.
    Real,
    /// Requires the lowered string type.
    String,
    /// Requires the lowered dynamic type.
    Dynamic,
    /// Requires a scalar comparable or dynamically erased type.
    Comparable,
}

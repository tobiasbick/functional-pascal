//! Typed three-address IR operations.

use crate::{
    EnumLayoutId, FieldId, FunctionId, GlobalId, IntrinsicId, LocalId, RecordLayoutId,
    ValueDefinition, ValueId, VariantId,
};

/// An instruction with an optional typed result definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
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
    /// Real addition.
    AddReal,
    /// A dynamically checked generic numeric addition.
    AddDynamic,
    /// Equality comparison of like-typed values.
    Equal,
    /// Integer less-than comparison.
    LessThanInteger,
    /// Boolean conjunction.
    AndBoolean,
    /// Boolean disjunction.
    OrBoolean,
    /// UTF-8 string concatenation.
    ConcatString,
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
                | Self::CallDirect { .. }
                | Self::CallValue { .. }
                | Self::LoadGlobal(_)
                | Self::MakeRecord { .. }
                | Self::LoadField { .. }
                | Self::MakeEnum { .. }
                | Self::TestVariant { .. }
                | Self::Intrinsic { .. }
                | Self::MakeClosure { .. }
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
        | BinaryOperation::DivideInteger => (TypeCategory::Integer, TypeCategory::Integer),
        BinaryOperation::AddReal => (TypeCategory::Real, TypeCategory::Real),
        BinaryOperation::AddDynamic => (TypeCategory::Dynamic, TypeCategory::Dynamic),
        BinaryOperation::Equal => (TypeCategory::Same, TypeCategory::Boolean),
        BinaryOperation::LessThanInteger => (TypeCategory::Integer, TypeCategory::Boolean),
        BinaryOperation::AndBoolean | BinaryOperation::OrBoolean => {
            (TypeCategory::Boolean, TypeCategory::Boolean)
        }
        BinaryOperation::ConcatString => (TypeCategory::String, TypeCategory::String),
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
}

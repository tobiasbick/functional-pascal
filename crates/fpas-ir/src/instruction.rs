//! Typed three-address IR operations.

mod operators;

pub use operators::{
    BinaryOperation, TypeCategory, UnaryOperation, binary_categories, unary_categories,
};

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
    /// Replaces a value below an indexed global aggregate while preserving snapshot semantics.
    StoreGlobalIndexPath {
        /// Target global.
        global: GlobalId,
        /// Snapshot read before the index expressions were evaluated.
        root: ValueId,
        /// Array indices or dictionary keys in outer-to-inner order.
        indexes: Vec<ValueId>,
        /// Replacement value.
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
    /// Removes and returns the last element of a local array with copy-on-write isolation.
    ArrayPop {
        /// Local array updated by the operation.
        local: LocalId,
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
    /// Replaces one indexed value directly in a mutable local collection.
    StoreLocalIndex {
        /// Mutable local array or dictionary updated by the operation.
        local: LocalId,
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
    /// Creates a reference to the variable stored in a capture cell.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    MakeCellReference(ValueId),
    /// Creates a reference to a mutable global slot.
    MakeGlobalReference(GlobalId),
    /// Narrows a reference to one record field.
    ReferenceField {
        /// Reference to a record value.
        reference: ValueId,
        /// Record layout of the referenced value.
        layout: RecordLayoutId,
        /// Field selected by the new reference.
        field: FieldId,
    },
    /// Narrows a reference to one array element; the index is fixed when created.
    ReferenceElement {
        /// Reference to an array value.
        reference: ValueId,
        /// Integer element index.
        index: ValueId,
    },
    /// Reads the current value through a reference.
    ReferenceRead(ValueId),
    /// Replaces the referenced value; the caller's variable changes immediately.
    ReferenceWrite {
        /// Target reference.
        reference: ValueId,
        /// Replacement value.
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
                | Self::ArrayPop { .. }
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
                | Self::MakeCellReference(_)
                | Self::MakeGlobalReference(_)
                | Self::ReferenceField { .. }
                | Self::ReferenceElement { .. }
                | Self::ReferenceRead(_)
                | Self::SpawnTask { .. }
        )
    }
}

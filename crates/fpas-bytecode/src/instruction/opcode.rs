//! Opcode names, forms, and stable wire discriminants.

use num_enum::TryFromPrimitive;

/// Physical payload layout declared by an opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionForm {
    /// Three 16-bit operands and an 8-bit auxiliary operand.
    Abc,
    /// One 16-bit operand and one 32-bit operand.
    Abx,
    /// One logical 48-bit operand.
    Ax,
}

/// Register-bytecode operation with a stable wire discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    /// Load a persistent constant.
    LoadConstant = 0,
    /// Load the Unit value.
    LoadUnit = 1,
    /// Copy a register value.
    Move = 2,
    /// Add integers.
    AddInteger = 3,
    /// Subtract integers.
    SubtractInteger = 4,
    /// Multiply integers.
    MultiplyInteger = 5,
    /// Divide integers.
    DivideInteger = 6,
    /// Calculate integer remainder.
    RemainderInteger = 7,
    /// Add real values.
    AddReal = 8,
    /// Subtract real values.
    SubtractReal = 9,
    /// Multiply real values.
    MultiplyReal = 10,
    /// Divide real values.
    DivideReal = 11,
    /// Negate an integer.
    NegateInteger = 12,
    /// Negate a real value.
    NegateReal = 13,
    /// Add dynamically typed numeric values.
    AddDynamic = 14,
    /// Subtract dynamically typed numeric values.
    SubtractDynamic = 15,
    /// Multiply dynamically typed numeric values.
    MultiplyDynamic = 16,
    /// Divide dynamically typed numeric values.
    DivideDynamic = 17,
    /// Negate a dynamically typed numeric value.
    NegateDynamic = 18,
    /// Test dynamic equality.
    EqualDynamic = 19,
    /// Test dynamic inequality.
    NotEqualDynamic = 20,
    /// Test dynamic less-than ordering.
    LessDynamic = 21,
    /// Test dynamic greater-than ordering.
    GreaterDynamic = 22,
    /// Test dynamic less-than-or-equal ordering.
    LessEqualDynamic = 23,
    /// Test dynamic greater-than-or-equal ordering.
    GreaterEqualDynamic = 24,
    /// Concatenate strings. Auxiliary 1 marks the left register as dead after this instruction,
    /// so its buffer may be reused for the result.
    ConcatString = 25,
    /// Shift an integer left.
    ShiftLeftInteger = 26,
    /// Shift an integer right.
    ShiftRightInteger = 27,
    /// Apply integer bitwise AND.
    BitAndInteger = 28,
    /// Apply integer bitwise OR.
    BitOrInteger = 29,
    /// Apply integer bitwise XOR.
    BitXorInteger = 30,
    /// Test integer equality.
    EqualInteger = 31,
    /// Test integer inequality.
    NotEqualInteger = 32,
    /// Test integer less-than ordering.
    LessInteger = 33,
    /// Test integer greater-than ordering.
    GreaterInteger = 34,
    /// Test integer less-than-or-equal ordering.
    LessEqualInteger = 35,
    /// Test integer greater-than-or-equal ordering.
    GreaterEqualInteger = 36,
    /// Test real equality.
    EqualReal = 37,
    /// Test real inequality.
    NotEqualReal = 38,
    /// Test real less-than ordering.
    LessReal = 39,
    /// Test real greater-than ordering.
    GreaterReal = 40,
    /// Test real less-than-or-equal ordering.
    LessEqualReal = 41,
    /// Test real greater-than-or-equal ordering.
    GreaterEqualReal = 42,
    /// Test string equality.
    EqualString = 43,
    /// Test string inequality.
    NotEqualString = 44,
    /// Test string less-than ordering.
    LessString = 45,
    /// Test string greater-than ordering.
    GreaterString = 46,
    /// Test string less-than-or-equal ordering.
    LessEqualString = 47,
    /// Test string greater-than-or-equal ordering.
    GreaterEqualString = 48,
    /// Test boolean equality.
    EqualBoolean = 49,
    /// Test boolean inequality.
    NotEqualBoolean = 50,
    /// Negate a boolean.
    NotBoolean = 51,
    /// Apply boolean AND to evaluated operands.
    AndBoolean = 52,
    /// Apply boolean OR to evaluated operands.
    OrBoolean = 53,
    /// Convert an integer to a real value.
    IntegerToReal = 54,
    /// Jump to an instruction in the current function.
    Jump = 55,
    /// Jump when a condition is false.
    BranchIfFalse = 56,
    /// Jump when a condition is true.
    BranchIfTrue = 57,
    /// Call a numeric function target.
    CallDirect = 58,
    /// Call a first-class function value.
    CallValue = 59,
    /// Construct a closure.
    MakeClosure = 60,
    /// Construct a mutable capture cell.
    MakeCell = 61,
    /// Read a mutable capture cell.
    CellRead = 62,
    /// Write a mutable capture cell.
    CellWrite = 63,
    /// Return from the current function.
    Return = 64,
    /// Raise a runtime panic value.
    Panic = 65,
    /// Load a global slot.
    LoadGlobal = 66,
    /// Store a global slot.
    StoreGlobal = 67,
    /// Construct an array from a contiguous register window.
    MakeArray = 68,
    /// Read a collection element.
    IndexGet = 69,
    /// Update a collection held in the first register operand.
    IndexSet = 70,
    /// Test collection membership.
    Contains = 71,
    /// Construct a dictionary from a contiguous register window.
    MakeDictionary = 72,
    /// Construct a record from a contiguous register window.
    MakeRecord = 73,
    /// Load a positional record field.
    LoadField = 74,
    /// Update a record held in the first register operand.
    StoreField = 75,
    /// Apply positional overrides to a record held in the first register operand.
    UpdateRecord = 76,
    /// Invoke a validated hosted intrinsic.
    Intrinsic = 77,
    /// Construct a Result.Ok value.
    MakeOk = 78,
    /// Construct a Result.Error value.
    MakeError = 79,
    /// Construct an Option.Some value.
    MakeSome = 80,
    /// Construct an Option.None value.
    MakeNone = 81,
    /// Test whether a Result is Ok.
    IsResultOk = 82,
    /// Test whether an Option is Some.
    IsOptionSome = 83,
    /// Unwrap a Result.Ok value.
    UnwrapOk = 84,
    /// Unwrap a Result.Error value.
    UnwrapError = 85,
    /// Unwrap an Option.Some value.
    UnwrapSome = 86,
    /// Construct an enum variant from a contiguous register window.
    MakeEnum = 87,
    /// Test an enum variant identifier.
    TestVariant = 88,
    /// Load positional associated data from an enum value.
    LoadEnumField = 89,
    /// Spawn a retained task.
    SpawnTask = 90,
    /// Spawn a detached task.
    SpawnDetachedTask = 91,
    /// Cooperatively yield the current task.
    Yield = 92,
    /// Append one value to an array using copy-on-write storage.
    ArrayPush = 93,
    /// Replace a value below an indexed global aggregate snapshot.
    StoreGlobalIndexPath = 94,
    /// Pops source array B in place and writes its last element to A. C and auxiliary are zero.
    ArrayPop = 95,
    /// Add a signed 16-bit immediate to an integer register.
    AddIntegerImm = 96,
    /// Divide an integer register by a signed 16-bit immediate.
    DivideIntegerImm = 97,
    /// Compare integers for equality and consume the following conditional branch.
    BranchIfEqualInteger = 98,
    /// Compare integers for inequality and consume the following conditional branch.
    BranchIfNotEqualInteger = 99,
    /// Compare integers for less-than and consume the following conditional branch.
    BranchIfLessInteger = 100,
    /// Compare integers for greater-than and consume the following conditional branch.
    BranchIfGreaterInteger = 101,
    /// Compare integers for less-or-equal and consume the following conditional branch.
    BranchIfLessEqualInteger = 102,
    /// Compare integers for greater-or-equal and consume the following conditional branch.
    BranchIfGreaterEqualInteger = 103,
    /// Check an inclusive integer loop boundary, then advance and consume the following jump.
    ForLoop = 104,
}

impl Opcode {
    /// Exhaustive opcode inventory used by format and verifier tests.
    pub const ALL: [Self; 105] = [
        Self::LoadConstant,
        Self::LoadUnit,
        Self::Move,
        Self::AddInteger,
        Self::SubtractInteger,
        Self::MultiplyInteger,
        Self::DivideInteger,
        Self::RemainderInteger,
        Self::AddReal,
        Self::SubtractReal,
        Self::MultiplyReal,
        Self::DivideReal,
        Self::NegateInteger,
        Self::NegateReal,
        Self::AddDynamic,
        Self::SubtractDynamic,
        Self::MultiplyDynamic,
        Self::DivideDynamic,
        Self::NegateDynamic,
        Self::EqualDynamic,
        Self::NotEqualDynamic,
        Self::LessDynamic,
        Self::GreaterDynamic,
        Self::LessEqualDynamic,
        Self::GreaterEqualDynamic,
        Self::ConcatString,
        Self::ShiftLeftInteger,
        Self::ShiftRightInteger,
        Self::BitAndInteger,
        Self::BitOrInteger,
        Self::BitXorInteger,
        Self::EqualInteger,
        Self::NotEqualInteger,
        Self::LessInteger,
        Self::GreaterInteger,
        Self::LessEqualInteger,
        Self::GreaterEqualInteger,
        Self::EqualReal,
        Self::NotEqualReal,
        Self::LessReal,
        Self::GreaterReal,
        Self::LessEqualReal,
        Self::GreaterEqualReal,
        Self::EqualString,
        Self::NotEqualString,
        Self::LessString,
        Self::GreaterString,
        Self::LessEqualString,
        Self::GreaterEqualString,
        Self::EqualBoolean,
        Self::NotEqualBoolean,
        Self::NotBoolean,
        Self::AndBoolean,
        Self::OrBoolean,
        Self::IntegerToReal,
        Self::Jump,
        Self::BranchIfFalse,
        Self::BranchIfTrue,
        Self::CallDirect,
        Self::CallValue,
        Self::MakeClosure,
        Self::MakeCell,
        Self::CellRead,
        Self::CellWrite,
        Self::Return,
        Self::Panic,
        Self::LoadGlobal,
        Self::StoreGlobal,
        Self::MakeArray,
        Self::IndexGet,
        Self::IndexSet,
        Self::Contains,
        Self::MakeDictionary,
        Self::MakeRecord,
        Self::LoadField,
        Self::StoreField,
        Self::UpdateRecord,
        Self::Intrinsic,
        Self::MakeOk,
        Self::MakeError,
        Self::MakeSome,
        Self::MakeNone,
        Self::IsResultOk,
        Self::IsOptionSome,
        Self::UnwrapOk,
        Self::UnwrapError,
        Self::UnwrapSome,
        Self::MakeEnum,
        Self::TestVariant,
        Self::LoadEnumField,
        Self::SpawnTask,
        Self::SpawnDetachedTask,
        Self::Yield,
        Self::ArrayPush,
        Self::StoreGlobalIndexPath,
        Self::ArrayPop,
        Self::AddIntegerImm,
        Self::DivideIntegerImm,
        Self::BranchIfEqualInteger,
        Self::BranchIfNotEqualInteger,
        Self::BranchIfLessInteger,
        Self::BranchIfGreaterInteger,
        Self::BranchIfLessEqualInteger,
        Self::BranchIfGreaterEqualInteger,
        Self::ForLoop,
    ];

    /// Return the physical payload form assigned to this opcode.
    #[must_use]
    pub const fn form(self) -> InstructionForm {
        match self {
            Self::LoadConstant
            | Self::Jump
            | Self::BranchIfFalse
            | Self::BranchIfTrue
            | Self::LoadGlobal
            | Self::StoreGlobal => InstructionForm::Abx,
            _ => InstructionForm::Abc,
        }
    }
}

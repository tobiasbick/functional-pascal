//! Portable register bytecode, executable metadata, and runtime values for Functional Pascal.

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

mod debug;
mod executable;
mod function;
mod instruction;
pub mod intrinsic;
pub mod limits;
mod metadata;
mod operand;
mod validate;
mod value;

pub use debug::{
    DebugBinding, DebugBindingKind, DebugCaptureKind, DebugCaptureSource, DebugEffectSet,
    DebugScope, DebugSourceLocation, DebugType, FunctionDebugInfo, FunctionEffectSummary,
    SequencePoint, analyze_debug_effects, intrinsic_debug_effects,
};
pub use executable::{Executable, VerifiedExecutable};
pub use fpas_diagnostics::SourceLocation;
pub use function::{CodeRange, FunctionFlags, FunctionInfo, ReturnConvention};
pub use instruction::{
    AbcOperands, AbxOperands, Instruction, InstructionError, InstructionForm, Opcode,
};
pub use intrinsic::{
    ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, ConvIntrinsic, DictIntrinsic, EnvIntrinsic,
    FsIntrinsic, Intrinsic, JsonIntrinsic, MathIntrinsic, OptionIntrinsic, ParseIntrinsic,
    PathIntrinsic, ProcIntrinsic, RandomIntrinsic, ResultIntrinsic, StrIntrinsic, TaskIntrinsic,
    TestIntrinsic, TimeIntrinsic, TomlIntrinsic,
};
pub use metadata::{
    Constant, EnumLayout, EnumVariant, GlobalInfo, GlobalInitializer, RecordField, RecordLayout,
    RecordMethod, RecordProperty, SourceMap, SourceRun, StringTable,
};
pub use operand::{
    ConstantId, DebugBindingId, DebugTypeId, EnumTypeId, EnumVariantId, FunctionId, GlobalId,
    InstructionAddress, IntrinsicId, NO_REGISTER, OperandError, RecordFieldId, RecordTypeId,
    Register, SourceId, StringId,
};
pub use validate::{ValidationError, ValidationErrorKind};
pub use value::{
    EnumValue, FunctionValue, RecordValue, RuntimeEnumLayout, RuntimeRecordLayout, SharedArray,
    SharedDict, SharedEnum, SharedFunction, SharedRecord, SharedStr, Value,
};

/// Persistent register instruction-set version recorded in compiled artifacts.
pub const BYTECODE_VERSION: u32 = 14;

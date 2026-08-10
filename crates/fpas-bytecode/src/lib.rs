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
    DebugBinding, DebugBindingKind, DebugScope, DebugSourceLocation, FunctionDebugInfo,
    SequencePoint,
};
pub use executable::{Executable, VerifiedExecutable};
pub use fpas_diagnostics::SourceLocation;
pub use function::{CodeRange, FunctionFlags, FunctionInfo, ReturnConvention};
pub use instruction::{
    AbcOperands, AbxOperands, Instruction, InstructionError, InstructionForm, Opcode,
};
pub use intrinsic::{
    ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, ConvIntrinsic, DictIntrinsic, EnvIntrinsic,
    FsIntrinsic, GraphIntrinsic, Intrinsic, JsonIntrinsic, MathIntrinsic, OptionIntrinsic,
    ParseIntrinsic, PathIntrinsic, ProcIntrinsic, RandomIntrinsic, ResultIntrinsic, StrIntrinsic,
    TaskIntrinsic, TestIntrinsic, TimeIntrinsic, TomlIntrinsic,
};
pub use metadata::{
    Constant, EnumLayout, EnumVariant, GlobalInfo, RecordField, RecordLayout, SourceMap, SourceRun,
    StringTable,
};
pub use operand::{
    ConstantId, EnumTypeId, EnumVariantId, FunctionId, GlobalId, InstructionAddress, IntrinsicId,
    NO_REGISTER, OperandError, RecordFieldId, RecordTypeId, Register, SourceId, StringId,
};
pub use validate::{ValidationError, ValidationErrorKind};
pub use value::{
    EnumValue, FunctionValue, RecordValue, RuntimeEnumLayout, RuntimeRecordLayout, SharedArray,
    SharedDict, SharedEnum, SharedFunction, SharedRecord, SharedStr, Value,
};

/// Persistent register instruction-set version recorded in compiled artifacts.
pub const BYTECODE_VERSION: u32 = 11;

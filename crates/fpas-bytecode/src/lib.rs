mod chunk;
mod chunk_validate;
mod executable;
mod function;
mod instruction;
pub mod intrinsic;
pub mod limits;
mod metadata;
mod op;
mod operand;
mod persistent_value;
mod validate;
mod value;

pub use chunk::{Chunk, ChunkError, MAX_CONSTANT_INDEX};
pub use chunk_validate::{ExecutableError, validate_executable};
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
pub use op::Op;
pub use operand::{
    ConstantId, EnumTypeId, EnumVariantId, FunctionId, GlobalId, InstructionAddress, IntrinsicId,
    NO_REGISTER, OperandError, RecordFieldId, RecordTypeId, Register, SourceId, StringId,
};
pub use persistent_value::{PersistentValue, PersistentValueError};
pub use validate::{ValidationError, ValidationErrorKind};
pub use value::{
    EnumValue, FunctionValue, RecordValue, SharedArray, SharedDict, SharedEnum, SharedFunction,
    SharedRecord, SharedStr, Value,
};

/// Persistent instruction-set version recorded in `.fpascu` identities.
pub const BYTECODE_VERSION: u32 = 1;

/// Inactive register instruction-set version, promoted at the production cutover.
pub const REGISTER_BYTECODE_VERSION: u32 = 2;

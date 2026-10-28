//! Portable register bytecode, executable metadata, and validation for Functional Pascal.

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
mod decoded;
mod executable;
mod function;
mod instruction;
pub mod intrinsic;
pub mod limits;
mod metadata;
mod operand;
mod validate;

pub use debug::{
    DebugBinding, DebugBindingKind, DebugCaptureKind, DebugCaptureSource, DebugEffectSet,
    DebugProjection, DebugProjectionKind, DebugScope, DebugSourceLocation, DebugType,
    FunctionDebugInfo, FunctionEffectSummary, MAX_DEBUG_PROJECTION_DEPTH,
    MAX_DEBUG_PROJECTION_NAME_BYTES, MAX_DEBUG_PROJECTION_NODES, SequencePoint,
    analyze_debug_effects, intrinsic_debug_effects,
};
pub use decoded::DecodedInstruction;
pub use executable::{Executable, ExecutableIdentity, FunctionTarget, VerifiedExecutable};
pub use fpas_diagnostics::SourceLocation;
pub use function::{CodeRange, FunctionFlags, FunctionInfo, ReturnConvention};
pub use instruction::{
    AbcOperands, AbxOperands, Instruction, InstructionError, InstructionForm, Opcode,
};
pub use intrinsic::{
    AcpIntrinsic, ArgsIntrinsic, ArrayIntrinsic, BitsIntrinsic, ConsoleIntrinsic, ConvIntrinsic,
    CryptoIntrinsic, DictIntrinsic, EncodingIntrinsic, EnvIntrinsic, FsIntrinsic, HttpIntrinsic,
    Intrinsic, IntrinsicOwner, JsonIntrinsic, MathIntrinsic, McpIntrinsic, NetIntrinsic,
    OptionIntrinsic, ParseIntrinsic, PathIntrinsic, ProcIntrinsic, RandomIntrinsic,
    ResultIntrinsic, ServerIntrinsic, StrIntrinsic, TaskIntrinsic, TestIntrinsic, TimeIntrinsic,
    TomlIntrinsic,
};
pub use metadata::{
    Constant, DistinctTypeName, EnumLayout, EnumVariant, ErrorCause, ErrorInstanceKey,
    ErrorVariant, GlobalInfo, GlobalInitializer, IdentityParameter, NominalKind,
    RecordConstructionInfo, RecordField, RecordLayout, RecordMethod, RecordTypeAlias, SourceMap,
    SourceRun, StringTable, TypeIdentity,
};
pub use operand::{
    ConstantId, DebugBindingId, DebugTypeId, EnumTypeId, EnumVariantId, FunctionId, GlobalId,
    InstructionAddress, IntrinsicId, NO_REGISTER, OperandError, RecordFieldId, RecordTypeId,
    Register, SourceId, StringId,
};
pub use validate::{ValidationError, ValidationErrorKind};
/// Persistent instruction-set and intrinsic ABI version recorded in compiled artifacts.
pub const BYTECODE_VERSION: u32 = 29;

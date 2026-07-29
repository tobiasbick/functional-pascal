mod chunk;
mod executable;
pub mod intrinsic;
mod op;
mod persistent_value;
mod value;

pub use chunk::{Chunk, ChunkError, MAX_CONSTANT_INDEX};
pub use executable::{ExecutableError, validate_executable};
pub use fpas_diagnostics::SourceLocation;
pub use intrinsic::{
    ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, ConvIntrinsic, DictIntrinsic, EnvIntrinsic,
    FsIntrinsic, GraphIntrinsic, Intrinsic, JsonIntrinsic, MathIntrinsic, OptionIntrinsic,
    ParseIntrinsic, PathIntrinsic, ProcIntrinsic, RandomIntrinsic, ResultIntrinsic, StrIntrinsic,
    TaskIntrinsic, TestIntrinsic, TimeIntrinsic, TomlIntrinsic,
};
pub use op::Op;
pub use persistent_value::{PersistentValue, PersistentValueError};
pub use value::{
    EnumValue, FunctionValue, RecordValue, SharedArray, SharedDict, SharedEnum, SharedFunction,
    SharedRecord, SharedStr, Value,
};

/// Persistent instruction-set version recorded in `.fpascu` identities.
pub const BYTECODE_VERSION: u32 = 1;

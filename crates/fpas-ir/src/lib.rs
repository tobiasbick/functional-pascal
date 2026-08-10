//! Target-independent typed control-flow IR for Functional Pascal.
//!
//! This crate models compiler-owned meaning before register-bytecode selection. It deliberately has
//! no dependency on the VM, bytecode codec, host ABI, or platform-specific runtime state.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod debug;
mod function;
mod id;
mod instruction;
mod program;
mod terminator;
pub mod validate;

pub use debug::{DebugBinding, DebugBindingKind, DebugScope, FunctionDebugInfo, SequencePoint};
pub use fpas_diagnostics::SourceSpan;
pub use function::{
    BasicBlock, BlockParameter, CaptureDeclaration, CaptureKind, Function, FunctionSignature,
    Local, ValueDefinition,
};
pub use id::{
    BlockId, EnumLayoutId, FieldId, FunctionId, GlobalId, IdConversionError, IntrinsicId, LocalId,
    RecordLayoutId, TypeId, ValueId, VariantId, checked_count,
};
pub use instruction::{BinaryOperation, Constant, Instruction, Operation, UnaryOperation};
pub use program::{
    EnumLayout, EnumVariant, Global, IntrinsicSignature, IrType, Program, RecordField,
    RecordLayout, RecordProperty, TypeDefinition,
};
pub use terminator::{BlockTarget, Terminator};

//! Validation tests for typed control-flow IR programs.

use fpas_ir::{
    BasicBlock, BinaryOperation, BlockId, BlockParameter, BlockTarget, CaptureDeclaration,
    CaptureKind, Constant, EnumLayout, EnumLayoutId, EnumVariant, FieldId, Function, FunctionId,
    FunctionSignature, Global, GlobalId, Instruction, IntrinsicId, IntrinsicSignature, IrType,
    Local, LocalId, Operation, Program, RecordField, RecordLayout, RecordLayoutId, SourceSpan,
    Terminator, TypeDefinition, TypeId, UnaryOperation, ValueDefinition, ValueId, VariantId,
    checked_count,
};

mod support {
    include!("validation/support.rs");
}

use support::*;

include!("validation/cases.rs");
include!("validation/cell_cases.rs");
include!("validation/p5_cases.rs");
include!("validation/table_cases.rs");

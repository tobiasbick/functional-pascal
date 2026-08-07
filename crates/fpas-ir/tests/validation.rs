use fpas_ir::{
    BasicBlock, BinaryOperation, BlockId, BlockParameter, BlockTarget, CaptureDeclaration,
    CaptureKind, Constant, EnumLayout, EnumLayoutId, EnumVariant, FieldId, Function, FunctionId,
    FunctionSignature, Global, GlobalId, Instruction, IntrinsicId, IntrinsicSignature, IrType,
    Local, LocalId, Operation, Program, RecordField, RecordLayout, RecordLayoutId, Terminator,
    TypeDefinition, TypeId, ValueDefinition, ValueId, VariantId, checked_count,
};

mod support {
    include!("validation/support.rs");
}

use support::*;

include!("validation/cases.rs");

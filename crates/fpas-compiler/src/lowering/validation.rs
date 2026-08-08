//! Source-aware diagnostics for typed register IR validation failures.

use fpas_ir::Program;

pub(super) fn source(
    ir: &Program,
    error: &fpas_ir::validate::ValidationError,
) -> Option<fpas_lexer::Span> {
    let function = ir
        .functions
        .iter()
        .find(|function| Some(function.id) == error.location.function)?;
    let block = function
        .blocks
        .iter()
        .find(|block| Some(block.id) == error.location.block)?;
    block
        .instructions
        .get(error.location.instruction?)?
        .source
        .map(|source| fpas_lexer::Span {
            offset: source.offset(),
            length: source.length(),
            line: source.line(),
            column: source.column(),
            source_id: source.source_id(),
        })
}

pub(super) fn context(ir: &Program, error: &fpas_ir::validate::ValidationError) -> String {
    let Some(function_id) = error.location.function else {
        return String::new();
    };
    let Some(function) = ir
        .functions
        .iter()
        .find(|function| function.id == function_id)
    else {
        return String::new();
    };
    let operation = error
        .location
        .block
        .and_then(|block| {
            function
                .blocks
                .iter()
                .find(|candidate| candidate.id == block)
        })
        .and_then(|block| {
            error
                .location
                .instruction
                .and_then(|index| block.instructions.get(index))
        })
        .map(|instruction| format!("; operation {:?}", instruction.operation))
        .unwrap_or_default();
    let types = match error.kind {
        fpas_ir::validate::ValidationErrorKind::OperandType {
            expected, actual, ..
        }
        | fpas_ir::validate::ValidationErrorKind::ReturnType { expected, actual }
        | fpas_ir::validate::ValidationErrorKind::BlockArgumentType { expected, actual } => {
            format!(
                "; expected type {}; actual type {}",
                type_name(ir, fpas_ir::TypeId::new(expected)),
                type_name(ir, fpas_ir::TypeId::new(actual))
            )
        }
        _ => String::new(),
    };
    format!("; function `{}`{operation}{types}", function.name)
}

fn type_name(ir: &Program, id: fpas_ir::TypeId) -> String {
    match ir.ty(id).map(|ty| &ty.kind) {
        Some(fpas_ir::IrType::Record(layout)) => ir
            .record_layout(*layout)
            .map(|layout| format!("record `{}`", layout.name))
            .unwrap_or_else(|| format!("record layout {}", layout.get())),
        Some(fpas_ir::IrType::Enum(layout)) => ir
            .enum_layout(*layout)
            .map(|layout| format!("enum `{}`", layout.name))
            .unwrap_or_else(|| format!("enum layout {}", layout.get())),
        Some(kind) => format!("{kind:?}"),
        None => format!("unknown type {}", id.get()),
    }
}

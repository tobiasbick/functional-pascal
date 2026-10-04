//! Verifier-safe placeholders replaced by linker-resolved functions.

use super::{CompileError, types, unsupported};
use fpas_ir::{
    BasicBlock, BlockId, Function, FunctionId, FunctionSignature, Instruction, Operation,
    Terminator, ValueDefinition, ValueId,
};

/// Build an unreachable panic body with the imported signature.
pub(super) fn imported_stub(
    id: FunctionId,
    name: &str,
    parameter_types: Vec<fpas_ir::TypeId>,
    result: fpas_ir::TypeId,
    span: fpas_lexer::Span,
) -> Result<Function, CompileError> {
    let parameters = parameter_types
        .iter()
        .copied()
        .enumerate()
        .map(|(index, ty)| {
            ValueId::try_from_index(index)
                .map(|id| ValueDefinition { id, ty })
                .map_err(|_| unsupported(span, "import parameter overflow"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let panic_id = ValueId::try_from_index(parameters.len())
        .map_err(|_| unsupported(span, "import stub value overflow"))?;
    Ok(Function {
        id,
        name: name.to_ascii_lowercase(),
        signature: FunctionSignature {
            parameters: parameter_types,
            result,
        },
        parameters,
        locals: Vec::new(),
        captures: Vec::new(),
        debug: fpas_ir::FunctionDebugInfo::default(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![Instruction {
                source: None,
                result: Some(ValueDefinition {
                    id: panic_id,
                    ty: types::STRING,
                }),
                operation: Operation::Const(fpas_ir::Constant::String(
                    "unlinked imported callable".to_string(),
                )),
            }],
            terminators: vec![Terminator::Panic(panic_id)],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    })
}

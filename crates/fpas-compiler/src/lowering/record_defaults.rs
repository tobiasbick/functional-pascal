//! Compiler-generated default routines reused by typed debugger construction.
//! See `docs/pascal/language/types/records.md` and `docs/pascal/tools/debugger.md`.

use fpas_ir::{Function, Terminator};
use fpas_parser::Expr;

use super::context::{FunctionInput, LoweringContext};
use super::types::TypeTable;
use crate::CompileError;

/// Lowers a default using the same declaration-scope path as ordinary construction.
pub(super) fn lower(
    input: FunctionInput<'_>,
    expression: &Expr,
) -> Result<(Function, TypeTable), CompileError> {
    let result_type = input.result;
    let mut context = LoweringContext::new(input)?;
    let result = context.lower_record_default(expression, result_type)?;
    context.terminate(Terminator::Return(Some(result)))?;
    context.finish(expression.span())
}

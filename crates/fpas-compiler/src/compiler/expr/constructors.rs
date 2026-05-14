use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::Op;
use fpas_parser::Expr;

use super::super::Compiler;

impl Compiler {
    /// Lower `Result` and `Option` constructor expressions.
    pub(super) fn compile_result_option_expr(
        &mut self,
        expr: &Expr,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::ResultOk(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeOk, Self::location_of(span));
            }
            Expr::ResultError(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeErr, Self::location_of(span));
            }
            Expr::OptionSome(inner, span) => {
                self.compile_expr(inner)?;
                self.emit(Op::MakeSome, Self::location_of(span));
            }
            Expr::OptionNone(span) => {
                self.emit(Op::MakeNone, Self::location_of(span));
            }
            other => {
                let span = other.span();
                return Err(internal_compiler_error(
                    "Compiler routed a non-Result/Option constructor to constructor lowering.",
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                ));
            }
        }

        Ok(())
    }
}

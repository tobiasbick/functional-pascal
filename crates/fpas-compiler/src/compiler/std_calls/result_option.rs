use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_result_option_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_RESULT_UNWRAP => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ResultUnwrap, location);
                Ok(true)
            }
            s::STD_RESULT_UNWRAP_OR => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::ResultUnwrapOr, location);
                Ok(true)
            }
            s::STD_RESULT_IS_OK => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ResultIsOk, location);
                Ok(true)
            }
            s::STD_RESULT_IS_ERR => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ResultIsError, location);
                Ok(true)
            }
            s::STD_OPTION_UNWRAP => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::OptionUnwrap, location);
                Ok(true)
            }
            s::STD_OPTION_UNWRAP_OR => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::OptionUnwrapOr, location);
                Ok(true)
            }
            s::STD_OPTION_IS_SOME => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::OptionIsSome, location);
                Ok(true)
            }
            s::STD_OPTION_IS_NONE => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::OptionIsNone, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

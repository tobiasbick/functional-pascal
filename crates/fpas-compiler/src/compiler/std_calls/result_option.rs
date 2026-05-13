//! Lowers `Std.Result` and `Std.Option` helpers to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/result.md`, `docs/pascal/std/option.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation};
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
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::Unwrap), location);
                Ok(true)
            }
            s::STD_RESULT_UNWRAP_OR => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::UnwrapOr), location);
                Ok(true)
            }
            s::STD_RESULT_IS_OK => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::IsOk), location);
                Ok(true)
            }
            s::STD_RESULT_IS_ERR => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::IsError), location);
                Ok(true)
            }
            s::STD_RESULT_MAP => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::Map), location);
                Ok(true)
            }
            s::STD_RESULT_AND_THEN => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::AndThen), location);
                Ok(true)
            }
            s::STD_RESULT_OR_ELSE => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Result(ResultIntrinsic::OrElse), location);
                Ok(true)
            }
            s::STD_OPTION_UNWRAP => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::Unwrap), location);
                Ok(true)
            }
            s::STD_OPTION_UNWRAP_OR => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::UnwrapOr), location);
                Ok(true)
            }
            s::STD_OPTION_IS_SOME => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::IsSome), location);
                Ok(true)
            }
            s::STD_OPTION_IS_NONE => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::IsNone), location);
                Ok(true)
            }
            s::STD_OPTION_MAP => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::Map), location);
                Ok(true)
            }
            s::STD_OPTION_AND_THEN => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::AndThen), location);
                Ok(true)
            }
            s::STD_OPTION_OR_ELSE => {
                self.expect_exact_args(name, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Option(OptionIntrinsic::OrElse), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

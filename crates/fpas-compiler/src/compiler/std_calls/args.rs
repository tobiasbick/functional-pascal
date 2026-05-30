//! Lowers `Std.Args` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/args.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{ArgsIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_args_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_ARGS_PARAM_COUNT => {
                self.expect_zero_args(s::STD_ARGS_PARAM_COUNT, args, location)?;
                self.emit_intrinsic(Intrinsic::Args(ArgsIntrinsic::ParamCount), location);
                Ok(true)
            }
            s::STD_ARGS_PARAM_STR => {
                self.expect_exact_args(s::STD_ARGS_PARAM_STR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Args(ArgsIntrinsic::ParamStr), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

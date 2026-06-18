//! Lowers `Std.Env` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/host/env.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{EnvIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Env` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_env_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_ENV_GET => {
                self.expect_exact_args(s::STD_ENV_GET, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Env(EnvIntrinsic::Get), location);
                Ok(true)
            }
            s::STD_ENV_EXISTS => {
                self.expect_exact_args(s::STD_ENV_EXISTS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Env(EnvIntrinsic::Exists), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

//! Lowers `Std.Proc` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/proc.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, ProcIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Proc` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_proc_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_PROC_RUN => {
                self.expect_exact_args(s::STD_PROC_RUN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Proc(ProcIntrinsic::Run), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

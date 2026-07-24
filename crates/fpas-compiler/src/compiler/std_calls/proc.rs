//! Lowers `Std.Proc` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/host/proc.md` (from the repository root).

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
            s::STD_PROC_CURRENT_EXECUTABLE => {
                self.expect_exact_args(s::STD_PROC_CURRENT_EXECUTABLE, 0, args, location)?;
                self.emit_intrinsic(Intrinsic::Proc(ProcIntrinsic::CurrentExecutable), location);
                Ok(true)
            }
            s::STD_PROC_RUN => {
                self.expect_exact_args(s::STD_PROC_RUN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Proc(ProcIntrinsic::Run), location);
                Ok(true)
            }
            s::STD_PROC_RUN_CAPTURE => {
                self.expect_exact_args(s::STD_PROC_RUN_CAPTURE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Proc(ProcIntrinsic::RunCapture), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

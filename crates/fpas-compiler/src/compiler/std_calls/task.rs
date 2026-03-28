use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_task_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TASK_WAIT => {
                self.expect_exact_args(s::STD_TASK_WAIT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TaskWait, location);
                Ok(true)
            }
            s::STD_TASK_WAIT_ALL => {
                self.expect_exact_args(s::STD_TASK_WAIT_ALL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TaskWaitAll, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

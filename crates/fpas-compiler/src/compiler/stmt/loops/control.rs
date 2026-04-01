use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};

impl Compiler {
    pub(in super::super) fn compile_break_stmt(
        &mut self,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.emit_loop_scope_pops(location);
        let jump = self.emit(Op::Jump(0), location);
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_patches.push(jump);
        }
        Ok(())
    }

    pub(in super::super) fn compile_continue_stmt(
        &mut self,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.emit_loop_scope_pops(location);
        let jump = self.emit(Op::Jump(0), location);
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.continue_patches.push(jump);
        }
        Ok(())
    }
}

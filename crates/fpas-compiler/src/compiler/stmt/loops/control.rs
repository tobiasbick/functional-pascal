use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::Op;

impl Compiler {
    pub(in super::super) fn compile_break_stmt(
        &mut self,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.emit_loop_scope_pops((line, column));
        let jump = self.emit(Op::Jump(0), (line, column));
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_patches.push(jump);
        }
        Ok(())
    }

    pub(in super::super) fn compile_continue_stmt(
        &mut self,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.emit_loop_scope_pops((line, column));
        let jump = self.emit(Op::Jump(0), (line, column));
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.continue_patches.push(jump);
        }
        Ok(())
    }
}

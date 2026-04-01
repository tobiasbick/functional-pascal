use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{Expr, Stmt};

impl Compiler {
    pub(in super::super) fn compile_while_stmt(
        &mut self,
        condition: &Expr,
        body: &Stmt,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.compile_expr(condition)?;
        let exit_jump = self.emit(Op::JumpIfFalse(0), location);
        self.compile_stmt(body)?;

        self.patch_continues(loop_start as u32, location)?;
        self.emit(Op::Jump(loop_start as u32), location);

        let after = self.chunk.len() as u32;
        self.patch_jump(exit_jump, after, location)?;
        self.patch_and_pop_breaks(after, location)?;
        Ok(())
    }

    pub(in super::super) fn compile_repeat_stmt(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.begin_scope();
        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        self.end_scope(location);

        let condition_start = self.chunk.len() as u32;
        self.patch_continues(condition_start, location)?;

        self.compile_expr(condition)?;
        self.emit(Op::JumpIfFalse(loop_start as u32), location);

        let after = self.chunk.len() as u32;
        self.patch_and_pop_breaks(after, location)?;
        Ok(())
    }
}

use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::Op;
use fpas_parser::Stmt;

impl Compiler {
    pub(in super::super) fn compile_if_stmt(
        &mut self,
        condition: &fpas_parser::Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.compile_expr(condition)?;
        let else_jump = self.emit(Op::JumpIfFalse(0), (line, column));
        self.compile_stmt(then_branch)?;

        if let Some(else_stmt) = else_branch {
            let end_jump = self.emit(Op::Jump(0), (line, column));
            let else_addr = self.chunk.len() as u32;
            self.patch_jump(else_jump, else_addr, (line, column))?;
            self.compile_stmt(else_stmt)?;
            let end_addr = self.chunk.len() as u32;
            self.patch_jump(end_jump, end_addr, (line, column))?;
        } else {
            let end_addr = self.chunk.len() as u32;
            self.patch_jump(else_jump, end_addr, (line, column))?;
        }
        Ok(())
    }
}

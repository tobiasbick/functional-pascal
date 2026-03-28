//! Statement lowering for block, assignment, return, panic, and call forms.
//!
//! **Documentation:** `docs/pascal/03-control-flow.md`, `docs/pascal/04-functions.md`, `docs/pascal/07-error-handling.md` (from the repository root).

use super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{Designator, Expr, Stmt, VarDef};

impl Compiler {
    pub(super) fn compile_block_stmt(
        &mut self,
        stmts: &[Stmt],
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.begin_scope();
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        self.end_scope((line, column));
        Ok(())
    }

    pub(super) fn compile_var_stmt(&mut self, var: &VarDef) -> Result<(), CompileError> {
        self.compile_expr(&var.value)?;
        self.add_local(&var.name);
        Ok(())
    }

    pub(super) fn compile_assign_stmt(
        &mut self,
        target: &Designator,
        value: &Expr,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.compile_designator_write(target, value, SourceLocation::new(line, column))
    }

    pub(super) fn compile_return_stmt(
        &mut self,
        expr: Option<&Expr>,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        if let Some(value) = expr {
            self.compile_expr(value)?;
        } else {
            self.emit(Op::Unit, (line, column));
        }
        self.emit(Op::Return, (line, column));
        Ok(())
    }

    pub(super) fn compile_panic_stmt(
        &mut self,
        expr: &Expr,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        self.compile_expr(expr)?;
        self.emit(Op::Panic, (line, column));
        Ok(())
    }

    pub(super) fn compile_call_stmt(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        let call_key = std::ptr::from_ref(designator) as usize;
        if let Some(qualified) = self.method_calls.get(&call_key).cloned() {
            self.compile_method_call(
                designator,
                &qualified,
                args,
                SourceLocation::new(line, column),
            )?;
            self.emit(Op::Pop, (line, column));
        } else {
            let name = Self::resolve_designator_name(designator);
            self.compile_call(&name, args, SourceLocation::new(line, column))?;
            self.emit(Op::Pop, (line, column));
        }
        Ok(())
    }
}

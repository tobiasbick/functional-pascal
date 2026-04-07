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
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.begin_scope();
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        self.end_scope(location);
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
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.compile_designator_write(target, value, location)
    }

    pub(super) fn compile_return_stmt(
        &mut self,
        expr: Option<&Expr>,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        if let Some(value) = expr {
            self.compile_expr(value)?;
        } else {
            self.emit(Op::Unit, location);
        }
        self.emit(Op::Return, location);
        Ok(())
    }

    pub(super) fn compile_panic_stmt(
        &mut self,
        expr: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.compile_expr(expr)?;
        self.emit(Op::Panic, location);
        Ok(())
    }

    pub(super) fn compile_call_stmt(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let call_key = fpas_sema::designator_lookup_key(designator);
        if let Some(qualified) = self.method_calls.get(&call_key).cloned() {
            self.compile_method_call(designator, &qualified, args, location)?;
            self.emit(Op::Pop, location);
        } else {
            let name = Self::resolve_designator_name(designator);
            self.compile_call(&name, args, location)?;
            self.emit(Op::Pop, location);
        }
        Ok(())
    }
}

//! Statement lowering for block, assignment, return, panic, and call forms.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`, `docs/pascal/language/functions/README.md`, `docs/pascal/language/error-handling/README.md` (from the repository root).

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
        self.add_local(&var.name, Self::location_of(&var.span))?;
        Ok(())
    }

    pub(super) fn compile_assign_stmt(
        &mut self,
        target: &Designator,
        value: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let key = fpas_sema::designator_lookup_key(target);
        if let Some(info) = self.property_writes.get(&key).cloned() {
            return self.compile_property_assignment(target, value, &info, location);
        }
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
        if let Some(target) = self.method_calls.get(&call_key).cloned() {
            match target {
                fpas_sema::MethodCallTarget::Instance {
                    qualified_name,
                    receiver_reads,
                } => {
                    self.compile_method_call(
                        designator,
                        &qualified_name,
                        &receiver_reads,
                        args,
                        location,
                    )?;
                    self.emit(Op::Pop, location);
                }
                fpas_sema::MethodCallTarget::Static(qualified) => {
                    self.compile_call(&qualified, args, location)?;
                    self.emit(Op::Pop, location);
                }
            }
        } else {
            let name = Self::resolve_designator_name(designator);
            self.compile_call(&name, args, location)?;
            self.emit(Op::Pop, location);
        }
        Ok(())
    }
}

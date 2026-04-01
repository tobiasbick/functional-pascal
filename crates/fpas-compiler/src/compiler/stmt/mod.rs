mod basic;
mod concurrency;
mod control_flow;
mod loops;

use crate::error::CompileError;
use fpas_parser::Stmt;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Block(stmts, span) => self.compile_block_stmt(stmts, Self::location_of(span)),
            Stmt::Var(v) | Stmt::MutableVar(v) => self.compile_var_stmt(v),
            Stmt::Assign {
                target,
                value,
                span,
            } => self.compile_assign_stmt(target, value, Self::location_of(span)),
            Stmt::Return(expr, span) => {
                self.compile_return_stmt(expr.as_ref(), Self::location_of(span))
            }
            Stmt::Panic(expr, span) => self.compile_panic_stmt(expr, Self::location_of(span)),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.compile_if_stmt(
                condition,
                then_branch,
                else_branch.as_deref(),
                Self::location_of(span),
            ),
            Stmt::While {
                condition,
                body,
                span,
            } => self.compile_while_stmt(condition, body, Self::location_of(span)),
            Stmt::Repeat {
                body,
                condition,
                span,
            } => self.compile_repeat_stmt(body, condition, Self::location_of(span)),
            Stmt::For {
                var_name,
                start,
                direction,
                end,
                body,
                span,
                ..
            } => self.compile_for_stmt(
                var_name,
                start,
                direction,
                end,
                body,
                Self::location_of(span),
            ),
            Stmt::ForIn {
                var_name,
                iterable,
                body,
                span,
                ..
            } => self.compile_for_in_stmt(var_name, iterable, body, Self::location_of(span)),
            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => self.compile_case_stmt(expr, arms, else_body.as_deref(), Self::location_of(span)),
            Stmt::Break(span) => self.compile_break_stmt(Self::location_of(span)),
            Stmt::Continue(span) => self.compile_continue_stmt(Self::location_of(span)),
            Stmt::Call {
                designator,
                args,
                span,
            } => self.compile_call_stmt(designator, args, Self::location_of(span)),
            Stmt::Go { expr, span } => self.compile_go_stmt(expr, *span),
        }
    }
}

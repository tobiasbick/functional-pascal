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
            Stmt::Block(stmts, span) => self.compile_block_stmt(stmts, span.line, span.column),
            Stmt::Var(v) | Stmt::MutableVar(v) => self.compile_var_stmt(v),
            Stmt::Assign {
                target,
                value,
                span,
            } => self.compile_assign_stmt(target, value, span.line, span.column),
            Stmt::Return(expr, span) => {
                self.compile_return_stmt(expr.as_ref(), span.line, span.column)
            }
            Stmt::Panic(expr, span) => self.compile_panic_stmt(expr, span.line, span.column),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => self.compile_if_stmt(
                condition,
                then_branch,
                else_branch.as_deref(),
                span.line,
                span.column,
            ),
            Stmt::While {
                condition,
                body,
                span,
            } => self.compile_while_stmt(condition, body, span.line, span.column),
            Stmt::Repeat {
                body,
                condition,
                span,
            } => self.compile_repeat_stmt(body, condition, span.line, span.column),
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
                (span.line, span.column),
            ),
            Stmt::ForIn {
                var_name,
                iterable,
                body,
                span,
                ..
            } => self.compile_for_in_stmt(var_name, iterable, body, span.line, span.column),
            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => self.compile_case_stmt(expr, arms, else_body.as_deref(), span.line, span.column),
            Stmt::Break(span) => self.compile_break_stmt(span.line, span.column),
            Stmt::Continue(span) => self.compile_continue_stmt(span.line, span.column),
            Stmt::Call {
                designator,
                args,
                span,
            } => self.compile_call_stmt(designator, args, span.line, span.column),
            Stmt::Go { expr, span } => self.compile_go_stmt(expr, *span),
        }
    }
}

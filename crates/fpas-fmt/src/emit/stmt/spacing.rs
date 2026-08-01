//! Blank-line policy between sibling statements.

use fpas_parser::{Expr, Stmt};

pub(super) fn needs_blank_line(previous: &Stmt, next: &Stmt) -> bool {
    !matches!(next, Stmt::Var(_) | Stmt::MutableVar(_)) && statement_ends_with_end(previous)
}

fn statement_ends_with_end(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Block(..)
        | Stmt::If { .. }
        | Stmt::Case { .. }
        | Stmt::For { .. }
        | Stmt::ForIn { .. }
        | Stmt::While { .. } => true,
        Stmt::Var(var) | Stmt::MutableVar(var) => expression_ends_with_end(&var.value),
        Stmt::Assign { value, .. } => expression_ends_with_end(value),
        Stmt::Return(Some(value), ..)
        | Stmt::Expression { expr: value, .. }
        | Stmt::Go { expr: value, .. } => expression_ends_with_end(value),
        Stmt::Return(None, ..)
        | Stmt::Panic(..)
        | Stmt::Repeat { .. }
        | Stmt::Break(..)
        | Stmt::Continue(..)
        | Stmt::Call { .. } => false,
    }
}

fn expression_ends_with_end(expr: &Expr) -> bool {
    match expr {
        Expr::RecordLiteral { .. } | Expr::RecordUpdate { .. } | Expr::Closure(..) => true,
        Expr::UnaryOp { operand, .. } | Expr::Try(operand, ..) | Expr::Go(operand, ..) => {
            expression_ends_with_end(operand)
        }
        Expr::BinaryOp { right, .. } => expression_ends_with_end(right),
        _ => false,
    }
}

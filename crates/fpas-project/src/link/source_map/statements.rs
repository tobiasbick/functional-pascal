use super::declarations::apply_var_def_source_id;
use super::expressions::{apply_designator_source_id, apply_expr_source_id};
use super::support::apply_span;
use super::types::apply_type_expr_source_id;

use fpas_parser::{CaseArm, CaseLabel, Stmt};

pub(super) fn apply_stmt_source_id(stmt: &mut Stmt, source_id: u32) {
    match stmt {
        Stmt::Block(stmts, span) => {
            for stmt in stmts {
                apply_stmt_source_id(stmt, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Var(var_def) | Stmt::MutableVar(var_def) => {
            apply_var_def_source_id(var_def, source_id)
        }
        Stmt::Assign {
            target,
            value,
            span,
        } => {
            apply_designator_source_id(target, source_id);
            apply_expr_source_id(value, source_id);
            apply_span(span, source_id);
        }
        Stmt::Return(expr, span) => {
            if let Some(expr) = expr {
                apply_expr_source_id(expr, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Panic(expr, span) => {
            apply_expr_source_id(expr, source_id);
            apply_span(span, source_id);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            apply_expr_source_id(condition, source_id);
            apply_stmt_source_id(then_branch, source_id);
            if let Some(else_branch) = else_branch {
                apply_stmt_source_id(else_branch, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Case {
            expr,
            arms,
            else_body,
            span,
        } => {
            apply_expr_source_id(expr, source_id);
            for arm in arms {
                apply_case_arm_source_id(arm, source_id);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    apply_stmt_source_id(stmt, source_id);
                }
            }
            apply_span(span, source_id);
        }
        Stmt::For {
            var_type,
            start,
            direction: _,
            end,
            body,
            span,
            ..
        } => {
            apply_type_expr_source_id(var_type, source_id);
            apply_expr_source_id(start, source_id);
            apply_expr_source_id(end, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::ForIn {
            var_type,
            iterable,
            body,
            span,
            ..
        } => {
            apply_type_expr_source_id(var_type, source_id);
            apply_expr_source_id(iterable, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            apply_expr_source_id(condition, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::Repeat {
            body,
            condition,
            span,
        } => {
            for stmt in body {
                apply_stmt_source_id(stmt, source_id);
            }
            apply_expr_source_id(condition, source_id);
            apply_span(span, source_id);
        }
        Stmt::Break(span) | Stmt::Continue(span) => apply_span(span, source_id),
        Stmt::Call {
            designator,
            args,
            span,
        } => {
            apply_designator_source_id(designator, source_id);
            for arg in args {
                apply_expr_source_id(arg, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Expression { expr, span } => {
            apply_expr_source_id(expr, source_id);
            apply_span(span, source_id);
        }
        Stmt::Go { expr, span } => {
            apply_expr_source_id(expr, source_id);
            apply_span(span, source_id);
        }
    }
}

fn apply_case_arm_source_id(arm: &mut CaseArm, source_id: u32) {
    for label in &mut arm.labels {
        apply_case_label_source_id(label, source_id);
    }
    if let Some(guard) = &mut arm.guard {
        apply_expr_source_id(guard, source_id);
    }
    apply_stmt_source_id(&mut arm.body, source_id);
    apply_span(&mut arm.span, source_id);
}

fn apply_case_label_source_id(label: &mut CaseLabel, source_id: u32) {
    match label {
        CaseLabel::Value { start, end, span } => {
            apply_expr_source_id(start, source_id);
            if let Some(end) = end {
                apply_expr_source_id(end, source_id);
            }
            apply_span(span, source_id);
        }
        CaseLabel::Destructure { span, .. } => apply_span(span, source_id),
    }
}

use super::declarations::apply_var_def_source_id;
use super::expressions::{apply_designator_source_id, apply_expr_source_id};
use super::support::apply_span;
use super::types::apply_type_expr_source_id;

use fpas_parser::{CaseArm, CaseLabel, Pattern, Stmt};

/// Applies the source file identity to a statement and its nested syntax.
pub(super) fn apply_stmt_source_id(stmt: &mut Stmt, source_id: u32) {
    match stmt {
        Stmt::Null(span) => apply_span(span, source_id),
        Stmt::Block(stmts, span) => {
            for stmt in stmts {
                apply_stmt_source_id(stmt, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Const(var_def) | Stmt::Var(var_def) => apply_var_def_source_id(var_def, source_id),
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
        Stmt::Go { expr, span } | Stmt::Discard { expr, span } => {
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

pub(super) fn apply_case_label_source_id(label: &mut CaseLabel, source_id: u32) {
    match label {
        CaseLabel::Value { start, end, span } => {
            apply_expr_source_id(start, source_id);
            if let Some(end) = end {
                apply_expr_source_id(end, source_id);
            }
            apply_span(span, source_id);
        }
        CaseLabel::Binding { span, .. } => apply_span(span, source_id),
        CaseLabel::Pattern(pattern) => apply_pattern_source_id(pattern, source_id),
    }
}

pub(super) fn apply_pattern_source_id(pattern: &mut Pattern, source_id: u32) {
    match pattern {
        Pattern::Binding { span, .. } | Pattern::Wildcard(span) => apply_span(span, source_id),
        Pattern::Value(expr) => apply_expr_source_id(expr, source_id),
        Pattern::Variant {
            constructor,
            fields,
            span,
        } => {
            apply_designator_source_id(constructor, source_id);
            for field in fields {
                if let Some((_, label_span)) = &mut field.label {
                    apply_span(label_span, source_id);
                }
                apply_pattern_source_id(&mut field.pattern, source_id);
            }
            apply_span(span, source_id);
        }
        Pattern::Destructure { payload, span, .. } => {
            if let Some(payload) = payload {
                apply_pattern_source_id(payload, source_id);
            }
            apply_span(span, source_id);
        }
    }
}

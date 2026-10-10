//! Recursive traversal of expression-owned comment anchors.

use fpas_parser::{Designator, DesignatorPart, Expr, FuncBody, PostfixOperation};

use super::{
    CollectedAnchors, collect_body, collect_closer, collect_decls, collect_stmts, push_span,
};
use crate::comments::anchors::span_end;
use fpas_lexer::Span;

/// Collects expression bodies and named endings without taking their owner's terminator.
pub(super) fn collect_expr(expr: &Expr, begins: &[usize], out: &mut CollectedAnchors) {
    match expr {
        Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
            collect_designator(designator, begins, out);
        }
        Expr::Call {
            designator, args, ..
        } => {
            collect_designator(designator, begins, out);
            for arg in args {
                collect_expr(arg, begins, out);
            }
        }
        Expr::UnaryOp { operand, .. }
        | Expr::Paren(operand, _)
        | Expr::ResultOk(operand, _)
        | Expr::ResultError(operand, _)
        | Expr::OptionSome(operand, _)
        | Expr::Try(operand, _)
        | Expr::Go(operand, _)
        | Expr::NamedArgument { value: operand, .. } => collect_expr(operand, begins, out),
        Expr::Is { value, pattern, .. } => {
            collect_expr(value, begins, out);
            super::collect_pattern(pattern, begins, out);
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_expr(left, begins, out);
            collect_expr(right, begins, out);
        }
        Expr::If {
            branches,
            else_value,
            else_span,
            span,
        } => {
            collect_expression_closer(*span, out);
            for (index, branch) in branches.iter().enumerate() {
                // Comments before the first branch belong to the enclosing statement.
                if index > 0 {
                    out.leading.push(branch.span.offset);
                }
                push_span(branch.span, out);
                collect_expr(&branch.condition, begins, out);
                collect_expr(&branch.value, begins, out);
            }
            out.leading.push(else_span.offset);
            push_span(*else_span, out);
            collect_expr(else_value, begins, out);
        }
        Expr::Case {
            selector,
            arms,
            else_arm,
            span,
        } => {
            collect_expression_closer(*span, out);
            collect_expr(selector, begins, out);
            for arm in arms {
                out.leading.push(arm.span.offset);
                push_span(arm.span, out);
                super::collect_case_labels(&arm.labels, begins, out);
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, begins, out);
                }
                collect_expr(&arm.value, begins, out);
            }
            if let Some(else_arm) = else_arm {
                out.leading.push(else_arm.span.offset);
                push_span(else_arm.span, out);
                collect_expr(&else_arm.value, begins, out);
            }
        }
        Expr::ArrayLiteral(elements, _) => {
            for element in elements {
                collect_expr(element, begins, out);
            }
        }
        Expr::DictLiteral(pairs, _) => {
            for (key, value) in pairs {
                collect_expr(key, begins, out);
                collect_expr(value, begins, out);
            }
        }
        Expr::RecordUpdate { base, fields, span } => {
            collect_expression_closer(*span, out);
            collect_expr(base, begins, out);
            for field in fields {
                out.leading.push(field.span.offset);
                push_span(field.span, out);
                collect_expr(&field.value, begins, out);
            }
        }
        Expr::Postfix {
            base, operations, ..
        } => {
            collect_expr(base, begins, out);
            for operation in operations {
                match operation {
                    PostfixOperation::Index { index, .. } => collect_expr(index, begins, out),
                    PostfixOperation::MethodCall { args, .. } => {
                        for arg in args {
                            collect_expr(arg, begins, out);
                        }
                    }
                    PostfixOperation::Field { .. } => {}
                }
            }
        }
        Expr::Closure(closure) => {
            collect_expression_closer(closure.span, out);
            let FuncBody::Block { nested, stmts } = &closure.body;
            collect_decls(nested, begins, out);
            collect_stmts(stmts, begins, out);
            collect_body(
                closure.span.offset,
                closure.span,
                nested,
                stmts,
                begins,
                out,
            );
        }
        Expr::Integer(..)
        | Expr::Real(..)
        | Expr::Str(..)
        | Expr::Bool(..)
        | Expr::OptionNone(..)
        | Expr::Error(..) => {}
    }
}

fn collect_expression_closer(span: Span, out: &mut CollectedAnchors) {
    collect_closer(span, out);
    // A statement or declaration ending here owns comments after its semicolon.
    if out
        .emission
        .iter()
        .any(|anchor| anchor.end == span_end(span))
    {
        return;
    }
    if let Some(&start) = out.closers.get(&span.offset) {
        out.emission.push(crate::comments::anchors::EmissionAnchor {
            start,
            end: span_end(span),
        });
    }
}

/// Collects expressions nested inside designator index operations.
pub(super) fn collect_designator(
    designator: &Designator,
    begins: &[usize],
    out: &mut CollectedAnchors,
) {
    for part in &designator.parts {
        if let DesignatorPart::Index(index, _) = part {
            collect_expr(index, begins, out);
        }
    }
}

//! Recursive traversal of expression-owned comment anchors.

use fpas_parser::{Designator, DesignatorPart, Expr, FuncBody, PostfixOperation};

use super::{CollectedAnchors, collect_body, collect_decls, collect_stmts};

pub(super) fn collect_expr(expr: &Expr, begins: &[usize], out: &mut CollectedAnchors) {
    match expr {
        Expr::InvalidRecord(span) => {
            out.obsolete_record.get_or_insert(*span);
        }
        Expr::If(decision) => {
            collect_expr(&decision.condition, begins, out);
            collect_expr(&decision.then_value, begins, out);
            for (condition, value) in &decision.elsif_values {
                collect_expr(condition, begins, out);
                collect_expr(value, begins, out);
            }
            collect_expr(&decision.else_value, begins, out);
        }
        Expr::Case(decision) => {
            collect_expr(&decision.value, begins, out);
            for arm in &decision.arms {
                super::push_span(arm.span, out);
                for label in &arm.labels {
                    label.visit_expressions(&mut |value| collect_expr(value, begins, out));
                }
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, begins, out);
                }
                collect_expr(&arm.body, begins, out);
            }
            if let Some(value) = &decision.else_value {
                collect_expr(value, begins, out);
            }
        }
        Expr::Designator(designator) | Expr::VarArgument(designator, _) => {
            collect_designator(designator, begins, out)
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
        | Expr::Go(operand, _) => collect_expr(operand, begins, out),
        Expr::BinaryOp { left, right, .. } => {
            collect_expr(left, begins, out);
            collect_expr(right, begins, out);
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
        Expr::RecordConstruction { fields, .. } => {
            for field in fields {
                collect_expr(&field.value, begins, out);
            }
        }
        Expr::RecordUpdate { base, fields, .. } => {
            collect_expr(base, begins, out);
            for field in fields {
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
                    PostfixOperation::Call { args, .. } => {
                        for arg in args {
                            collect_expr(arg, begins, out);
                        }
                    }
                    PostfixOperation::Field { .. } => {}
                }
            }
        }
        Expr::Closure(closure) => {
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

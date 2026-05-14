use super::support::apply_span;

use fpas_parser::{Designator, DesignatorPart, Expr, FieldInit};

pub(super) fn apply_expr_source_id(expr: &mut Expr, source_id: u32) {
    match expr {
        Expr::Integer(_, span)
        | Expr::Real(_, span)
        | Expr::Str(_, span)
        | Expr::Bool(_, span)
        | Expr::Paren(_, span)
        | Expr::ArrayLiteral(_, span)
        | Expr::DictLiteral(_, span)
        | Expr::ResultOk(_, span)
        | Expr::ResultError(_, span)
        | Expr::OptionSome(_, span)
        | Expr::OptionNone(span)
        | Expr::Try(_, span)
        | Expr::Go(_, span)
        | Expr::Error(span) => {
            apply_span(span, source_id);
        }
        Expr::Designator(designator) => apply_designator_source_id(designator, source_id),
        Expr::Call {
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
        Expr::UnaryOp { operand, span, .. } => {
            apply_expr_source_id(operand, source_id);
            apply_span(span, source_id);
        }
        Expr::BinaryOp {
            left, right, span, ..
        } => {
            apply_expr_source_id(left, source_id);
            apply_expr_source_id(right, source_id);
            apply_span(span, source_id);
        }
        Expr::RecordLiteral { fields, span } => {
            for field in fields {
                apply_field_init_source_id(field, source_id);
            }
            apply_span(span, source_id);
        }
        Expr::RecordUpdate { base, fields, span } => {
            apply_expr_source_id(base, source_id);
            for field in fields {
                apply_field_init_source_id(field, source_id);
            }
            apply_span(span, source_id);
        }
    }

    match expr {
        Expr::Paren(inner, _)
        | Expr::ResultOk(inner, _)
        | Expr::ResultError(inner, _)
        | Expr::OptionSome(inner, _)
        | Expr::Try(inner, _)
        | Expr::Go(inner, _) => {
            apply_expr_source_id(inner, source_id);
        }
        Expr::ArrayLiteral(elements, _) => {
            for element in elements {
                apply_expr_source_id(element, source_id);
            }
        }
        Expr::DictLiteral(entries, _) => {
            for (key, value) in entries {
                apply_expr_source_id(key, source_id);
                apply_expr_source_id(value, source_id);
            }
        }
        _ => {}
    }
}

pub(super) fn apply_designator_source_id(designator: &mut Designator, source_id: u32) {
    for part in &mut designator.parts {
        match part {
            DesignatorPart::Ident(_, span) => apply_span(span, source_id),
            DesignatorPart::Index(expr, span) => {
                apply_expr_source_id(expr, source_id);
                apply_span(span, source_id);
            }
        }
    }
    apply_span(&mut designator.span, source_id);
}

fn apply_field_init_source_id(field: &mut FieldInit, source_id: u32) {
    apply_expr_source_id(&mut field.value, source_id);
    apply_span(&mut field.span, source_id);
}

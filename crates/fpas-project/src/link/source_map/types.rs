use super::support::{apply_qualified_id_source_id, apply_span};

use fpas_parser::{FormalParam, TypeExpr};

pub(super) fn apply_formal_param_source_id(param: &mut FormalParam, source_id: u32) {
    apply_type_expr_source_id(&mut param.type_expr, source_id);
    apply_span(&mut param.span, source_id);
}

pub(super) fn apply_type_expr_source_id(type_expr: &mut TypeExpr, source_id: u32) {
    match type_expr {
        TypeExpr::Named { id, span } => {
            apply_qualified_id_source_id(id, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::Array(inner, span)
        | TypeExpr::Option {
            inner_type: inner,
            span,
        } => {
            apply_type_expr_source_id(inner, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::FunctionType {
            params,
            return_type,
            span,
        } => {
            for param in params {
                apply_formal_param_source_id(param, source_id);
            }
            apply_type_expr_source_id(return_type, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::ProcedureType { params, span } => {
            for param in params {
                apply_formal_param_source_id(param, source_id);
            }
            apply_span(span, source_id);
        }
        TypeExpr::Result {
            ok_type,
            err_type,
            span,
        } => {
            apply_type_expr_source_id(ok_type, source_id);
            apply_type_expr_source_id(err_type, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::Dict {
            key_type,
            value_type,
            span,
        } => {
            apply_type_expr_source_id(key_type, source_id);
            apply_type_expr_source_id(value_type, source_id);
            apply_span(span, source_id);
        }
    }
}

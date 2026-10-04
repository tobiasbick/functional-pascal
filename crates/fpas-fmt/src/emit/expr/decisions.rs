//! Value decisions retain named expression closers and arm separators.

use super::{Emitter, emit_expr};
use crate::comments::{CommentMap, emit_leading_comments};
use fpas_parser::{CaseExpr, IfExpr};

pub(super) fn emit_if_expression(emitter: &mut Emitter, decision: &IfExpr, comments: &CommentMap) {
    emitter.write("if ");
    emit_expr(emitter, &decision.condition, 0, comments);
    emitter.write(" then ");
    emit_expr(emitter, &decision.then_value, 0, comments);
    for (condition, value) in &decision.elsif_values {
        emitter.write(" elsif ");
        emit_expr(emitter, condition, 0, comments);
        emitter.write(" then ");
        emit_expr(emitter, value, 0, comments);
    }
    emitter.write(" else ");
    emit_expr(emitter, &decision.else_value, 0, comments);
    emitter.write(" end if");
}

pub(super) fn emit_case_expression(
    emitter: &mut Emitter,
    decision: &CaseExpr,
    comments: &CommentMap,
) {
    emitter.write("case ");
    emit_expr(emitter, &decision.value, 0, comments);
    emitter.write(" of\n");
    emitter.with_indent(|inner| {
        for arm in &decision.arms {
            emit_leading_comments(inner, comments, arm.span.offset, false);
            inner.write_indent();
            inner.write("when ");
            super::super::stmt::emit_case_labels(inner, &arm.labels, comments);
            if let Some(guard) = &arm.guard {
                inner.write(" if ");
                emit_expr(inner, guard, 0, comments);
            }
            inner.write(": ");
            emit_expr(inner, &arm.body, 0, comments);
            inner.write(";\n");
        }
        if let Some(value) = &decision.else_value {
            inner.write_indent();
            inner.write("else ");
            emit_expr(inner, value, 0, comments);
            inner.write(";\n");
        }
    });
    emitter.write_indent();
    emitter.write("end case");
}

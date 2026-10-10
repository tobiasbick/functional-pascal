//! `case` expressions: one arm per line, indented below the enclosing statement.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md` and
//! `docs/pascal/language/control-flow/case-of-intro.md`.

use fpas_lexer::Span;
use fpas_parser::{CaseExprArm, CaseExprElse, Expr};

use super::super::Emitter;
use super::super::stmt::emit_case_labels;
use super::{emit_expr, emit_expression_end};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};

/// Emits `case Selector of`, each arm and the `else` arm on its own line, and `end case`.
pub(super) fn emit_case_expression(
    emitter: &mut Emitter,
    selector: &Expr,
    arms: &[CaseExprArm],
    else_arm: Option<&CaseExprElse>,
    span: Span,
    comments: &CommentMap,
) {
    emitter.write("case ");
    emit_expr(emitter, selector, 0, comments);
    emitter.write(" of");
    emitter.write_line_end();
    emitter.with_indent(|inner| {
        for arm in arms {
            emit_leading_comments(inner, comments, arm.span.offset, false);
            inner.write_current_indent();
            inner.write("when ");
            emit_case_labels(inner, &arm.labels, comments);
            if let Some(guard) = &arm.guard {
                inner.write(" if ");
                emit_expr(inner, guard, 0, comments);
            }
            inner.write(": ");
            emit_expr(inner, &arm.value, 0, comments);
            finish_arm(inner, arm.span.offset, comments);
        }
        if let Some(else_arm) = else_arm {
            emit_leading_comments(inner, comments, else_arm.span.offset, false);
            inner.write_current_indent();
            inner.write("else ");
            emit_expr(inner, &else_arm.value, 0, comments);
            finish_arm(inner, else_arm.span.offset, comments);
        }
    });
    emit_expression_end(
        emitter,
        comments.closer_anchor(span.offset),
        "end case",
        comments,
    );
}

/// Terminates an arm value with `;` and keeps its trailing comments on the line.
fn finish_arm(emitter: &mut Emitter, anchor: usize, comments: &CommentMap) {
    emitter.write(";");
    emit_trailing_comments(emitter, comments, anchor);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

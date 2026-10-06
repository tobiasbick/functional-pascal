//! Record-update expressions and their field and ending comments.
//!
//! **Documentation:** `docs/pascal/language/types/record-update.md`.

use fpas_parser::{Expr, FieldInit};

use super::super::Emitter;
use super::{emit_expr, emit_expression_end};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};

/// Emits an update's named ending, preserving compact output when it has no comments.
pub(super) fn emit_record_update(
    emitter: &mut Emitter,
    base: &Expr,
    fields: &[FieldInit],
    owner_start: usize,
    comments: &CommentMap,
) {
    emit_expr(emitter, base, 0, comments);
    let closer = comments.closer_anchor(owner_start);
    let has_comments = fields.iter().any(|field| {
        !comments.leading_at(field.span.offset).is_empty()
            || !comments.trailing_at(field.span.offset).is_empty()
    }) || closer.is_some_and(|anchor| {
        !comments.leading_at(anchor).is_empty() || !comments.trailing_at(anchor).is_empty()
    });
    if !has_comments {
        emitter.write(" with ");
        super::literal::emit_record_field_inits(emitter, fields, comments);
        if !fields.is_empty() {
            emitter.write("; ");
        }
        emitter.write("end with");
        return;
    }

    emitter.write(" with\n");
    emitter.with_indent(|inner| {
        for field in fields {
            emit_leading_comments(inner, comments, field.span.offset, false);
            inner.write_current_indent();
            inner.write(&field.name);
            inner.write(" := ");
            emit_expr(inner, &field.value, 0, comments);
            inner.write(";");
            emit_trailing_comments(inner, comments, field.span.offset);
            if !inner.ends_with_newline() {
                inner.write_line_end();
            }
        }
    });
    emit_expression_end(emitter, closer, "end with", comments);
}

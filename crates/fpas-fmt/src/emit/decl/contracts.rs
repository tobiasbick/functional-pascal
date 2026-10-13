//! Formatting of routine-entry and return clauses.

use super::super::{Emitter, expr::emit_expr};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};
use fpas_parser::{EnsuresClause, RequiresClause};

/// Emit clauses between the routine header and its declarations or body.
pub(super) fn emit_requires_clauses(
    emitter: &mut Emitter,
    clauses: &[RequiresClause],
    comments: &CommentMap,
) {
    for clause in clauses {
        emit_leading_comments(emitter, comments, clause.span.offset, false);
        emitter.write_current_indent();
        emitter.write("requires ");
        emit_expr(emitter, &clause.condition, 0, comments);
        emitter.write(";");
        emit_trailing_comments(emitter, comments, clause.span.offset);
        if !emitter.ends_with_newline() {
            emitter.write_line_end();
        }
    }
}

/// Emit postconditions with an optional clause-local return name and its required colon.
pub(super) fn emit_ensures_clauses(
    emitter: &mut Emitter,
    clauses: &[EnsuresClause],
    comments: &CommentMap,
) {
    for clause in clauses {
        emit_leading_comments(emitter, comments, clause.span.offset, false);
        emitter.write_current_indent();
        emitter.write("ensures ");
        if let Some((name, _)) = &clause.result_name {
            emitter.write("(");
            emitter.write(name);
            emitter.write("): ");
        }
        emit_expr(emitter, &clause.condition, 0, comments);
        emitter.write(";");
        emit_trailing_comments(emitter, comments, clause.span.offset);
        if !emitter.ends_with_newline() {
            emitter.write_line_end();
        }
    }
}

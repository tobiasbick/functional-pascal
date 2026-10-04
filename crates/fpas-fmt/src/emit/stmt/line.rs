//! Statement line layout helpers and simple `var` statements.

use fpas_parser::{BindingDef, Stmt};

use crate::comments::{CommentMap, emit_trailing_comments, stmt_start};

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::super::types::emit_type_expr;

pub(super) fn emit_var_stmt(
    emitter: &mut Emitter,
    keyword: &str,
    var: &BindingDef,
    comments: &CommentMap,
) {
    write_indented(emitter);
    emitter.write(keyword);
    emitter.write(" ");
    emitter.write(&var.name);
    if let Some(annotation) = &var.type_expr {
        emitter.write(": ");
        emit_type_expr(emitter, annotation);
    }
    emitter.write(" := ");
    emit_expr(emitter, &var.value, 0, comments);
    finish_stmt_line_at(emitter, comments, var.span.offset);
}
pub(super) fn write_indented(emitter: &mut Emitter) {
    emitter.write_current_indent();
}

pub(super) fn finish_stmt_line(emitter: &mut Emitter, comments: &CommentMap, stmt: &Stmt) {
    finish_stmt_line_at(emitter, comments, stmt_start(stmt));
}

fn finish_stmt_line_at(emitter: &mut Emitter, comments: &CommentMap, anchor_start: usize) {
    emitter.write(";");
    emit_trailing_comments(emitter, comments, anchor_start);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

pub(super) fn finish_stmt_after_newline(emitter: &mut Emitter, comments: &CommentMap, stmt: &Stmt) {
    emitter.remove_line_end();
    emitter.write(";");
    emit_trailing_comments(emitter, comments, stmt_start(stmt));
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

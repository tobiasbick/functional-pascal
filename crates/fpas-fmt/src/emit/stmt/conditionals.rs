//! Conditional clauses, scoped body lists, and named control endings.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#blocks-begin--end`.

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::line::write_indented;
use crate::comments::{CommentMap, emit_leading_comments, stmt_start};
use fpas_parser::Stmt;

/// Emits an `if` chain, preserving an explicitly nested conditional in `else`.
pub(super) fn emit_if(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    emit_clause(emitter, stmt, "if", comments);
    emit_control_end(emitter, stmt, "end if", comments);
}

fn emit_clause(emitter: &mut Emitter, stmt: &Stmt, keyword: &str, comments: &CommentMap) {
    let Stmt::If {
        condition,
        then_branch,
        else_branch,
        ..
    } = stmt
    else {
        return;
    };
    write_indented(emitter);
    emitter.write(keyword);
    emitter.write(" ");
    emit_expr(emitter, condition, 0, comments);
    emitter.write(" then\n");
    emit_control_body(emitter, then_branch, comments);
    match else_branch.as_deref() {
        Some(branch @ Stmt::If { .. }) => {
            emit_leading_comments(emitter, comments, stmt_start(branch), false);
            emit_clause(emitter, branch, "elsif", comments);
        }
        Some(branch) => {
            emit_leading_comments(emitter, comments, stmt_start(branch), false);
            emitter.writeln("else");
            emit_body_statements(emitter, branch, comments);
        }
        None => {}
    }
}

/// Emits the implicit body scope, leaving explicit inner blocks intact.
pub(super) fn emit_control_body(emitter: &mut Emitter, body: &Stmt, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, stmt_start(body), false);
    emit_body_statements(emitter, body, comments);
}

fn emit_body_statements(emitter: &mut Emitter, body: &Stmt, comments: &CommentMap) {
    emitter.with_indent(|inner| match body {
        Stmt::Block(statements, _) => super::emit_stmts_in_block(inner, statements, comments),
        other => super::emit_stmt_in_block(inner, other, comments),
    });
}

/// Emits leading comments attached to a named ending before that ending.
pub(super) fn emit_control_end(
    emitter: &mut Emitter,
    stmt: &Stmt,
    text: &str,
    comments: &CommentMap,
) {
    if let Some(anchor) = comments.closer_anchor(stmt_start(stmt)) {
        emit_leading_comments(emitter, comments, anchor, false);
    }
    emitter.writeln(text);
}

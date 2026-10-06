//! Counting, collection, while, and repeat loop emission.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#blocks-begin--end`.

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::super::types::emit_type_expr;
use super::line::write_indented;
use crate::comments::CommentMap;
use fpas_parser::{ForDirection, Stmt};

/// Emits counting and collection loops with scoped bodies and `end for`.
pub(super) fn emit_for(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    match stmt {
        Stmt::For {
            var_name,
            var_type,
            start,
            direction,
            end,
            body,
            ..
        } => {
            write_indented(emitter);
            emitter.write("for ");
            emitter.write(var_name);
            emitter.write(": ");
            emit_type_expr(emitter, var_type);
            emitter.write(" := ");
            emit_expr(emitter, start, 0, comments);
            emitter.write(" ");
            emitter.write(match direction {
                ForDirection::To => "to",
                ForDirection::Downto => "downto",
            });
            emitter.write(" ");
            emit_expr(emitter, end, 0, comments);
            emitter.write(" do\n");
            super::conditionals::emit_control_body(emitter, body, comments);
            super::conditionals::emit_control_end(emitter, stmt, "end for", comments);
        }
        Stmt::ForIn {
            var_name,
            var_type,
            iterable,
            body,
            ..
        } => {
            write_indented(emitter);
            emitter.write("for ");
            emitter.write(var_name);
            emitter.write(": ");
            emit_type_expr(emitter, var_type);
            emitter.write(" in ");
            emit_expr(emitter, iterable, 0, comments);
            emitter.write(" do\n");
            super::conditionals::emit_control_body(emitter, body, comments);
            super::conditionals::emit_control_end(emitter, stmt, "end for", comments);
        }
        _ => {}
    }
}

/// Emits a while loop with its scoped body and `end while`.
pub(super) fn emit_while(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::While {
        condition, body, ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write("while ");
    emit_expr(emitter, condition, 0, comments);
    emitter.write(" do\n");
    super::conditionals::emit_control_body(emitter, body, comments);
    super::conditionals::emit_control_end(emitter, stmt, "end while", comments);
}

/// Emits terminated repeat-body statements before the until condition.
pub(super) fn emit_repeat(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::Repeat {
        body, condition, ..
    } = stmt
    else {
        return;
    };

    emitter.writeln("repeat");
    emitter.with_indent(|inner| super::emit_stmts_in_block(inner, body, comments));
    write_indented(emitter);
    emitter.write("until ");
    emit_expr(emitter, condition, 0, comments);
}

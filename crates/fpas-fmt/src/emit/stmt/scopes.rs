//! Task scope emission.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

use fpas_parser::Stmt;

use crate::comments::CommentMap;

use super::Emitter;

/// Emits `scope`, its indented statements, and the named ending.
pub(super) fn emit_scope(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::Scope { body, .. } = stmt else {
        return;
    };
    emitter.writeln("scope");
    emitter.with_indent(|inner| super::emit_stmts_in_block(inner, body, comments));
    super::conditionals::emit_control_end(emitter, stmt, "end scope", comments);
}

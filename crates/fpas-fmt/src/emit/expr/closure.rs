//! Compact emission of anonymous function / procedure literals (closures).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use fpas_parser::{FormalParam, FuncBody, TypeExpr};

use crate::comments::CommentMap;

use super::super::Emitter;
use super::super::decl::emit_decl;
use super::super::stmt::emit_stmts_in_block;
use super::super::types::{emit_formal_params_in_parens, emit_type_expr};

/// Emit `function(…) : T begin … end` or `procedure(…) begin … end`.
pub(super) fn emit_closure(
    emitter: &mut Emitter,
    is_function: bool,
    params: &[FormalParam],
    return_type: &Option<TypeExpr>,
    body: &FuncBody,
    owner_start: usize,
    comments: &CommentMap,
) {
    if is_function {
        emit_formal_params_in_parens(emitter, "function(", params, "");
        emitter.write(": ");
        match return_type {
            Some(ty) => emit_type_expr(emitter, ty),
            None => emitter.write("<error>"),
        }
    } else {
        emit_formal_params_in_parens(emitter, "procedure(", params, "");
    }
    emit_closure_body(emitter, owner_start, body, comments);
}

fn emit_closure_body(
    emitter: &mut Emitter,
    owner_start: usize,
    body: &FuncBody,
    comments: &CommentMap,
) {
    let FuncBody::Block { nested, stmts } = body;
    let body_anchor = comments.body_anchor(owner_start);
    let has_body_comments =
        body_anchor.is_some_and(|anchor| !comments.leading_at(anchor).is_empty());
    if nested.is_empty() && !has_body_comments {
        emitter.write(" ");
    } else if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
    for decl in nested {
        emit_decl(emitter, decl, comments);
    }
    if let Some(anchor) = body_anchor {
        crate::comments::emit_leading_comments(emitter, comments, anchor, false);
    }
    if emitter.ends_with_newline() {
        emitter.write_current_indent();
    }
    emitter.write("begin");
    if stmts.is_empty() && nested.is_empty() {
        emitter.write(" end");
        return;
    }
    emitter.write("\n");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts, comments));
    emitter.write_current_indent();
    emitter.write("end");
}

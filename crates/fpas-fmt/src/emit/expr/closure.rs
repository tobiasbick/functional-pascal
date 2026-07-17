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
    emitter.write(" ");
    emit_closure_body(emitter, body);
}

fn emit_closure_body(emitter: &mut Emitter, body: &FuncBody) {
    let FuncBody::Block { nested, stmts } = body;
    let comments = CommentMap::default();
    for decl in nested {
        emit_decl(emitter, decl, &comments);
    }
    emitter.write("begin");
    if stmts.is_empty() && nested.is_empty() {
        emitter.write(" end");
        return;
    }
    emitter.write("\n");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts, &comments));
    emitter.write_current_indent();
    emitter.write("end");
}

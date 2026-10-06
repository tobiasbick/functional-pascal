//! Named routine headers, bodies, and declaration closers.

use super::super::Emitter;
use super::super::stmt::emit_stmts_in_block;
use super::super::types::{emit_formal_params_in_parens, emit_type_expr, format_type_params};
use super::item::{emit_decl, emit_visibility};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl};

/// Emits a named function declaration with its matching ending.
pub(super) fn emit_function_decl(
    emitter: &mut Emitter,
    function: &FunctionDecl,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    emit_visibility(emitter, function.visibility);
    emit_function_header(
        emitter,
        &function.name,
        &function.type_params,
        &function.params,
    );
    emitter.write(": ");
    emit_type_expr(emitter, &function.return_type);
    finish_routine_header_line(emitter, comments, function.span.offset);
    emit_func_body(
        emitter,
        function.span.offset,
        &function.body,
        "function",
        comments,
    );
}

/// Emits a named procedure declaration with its matching ending.
pub(super) fn emit_procedure_decl(
    emitter: &mut Emitter,
    procedure: &ProcedureDecl,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    emit_visibility(emitter, procedure.visibility);
    emit_procedure_header(
        emitter,
        &procedure.name,
        &procedure.type_params,
        &procedure.params,
    );
    finish_routine_header_line(emitter, comments, procedure.span.offset);
    emit_func_body(
        emitter,
        procedure.span.offset,
        &procedure.body,
        "procedure",
        comments,
    );
}

/// Emits a function signature before its declaration terminator.
pub(super) fn emit_function_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("function {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

/// Emits a procedure signature before its declaration terminator.
pub(super) fn emit_procedure_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("procedure {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

/// Emits nested declarations, body statements, and a named routine ending.
pub(super) fn emit_func_body(
    emitter: &mut Emitter,
    owner_start: usize,
    body: &FuncBody,
    kind: &str,
    comments: &CommentMap,
) {
    let FuncBody::Block { nested, stmts } = body;
    for decl in nested {
        emit_decl(emitter, decl, comments);
    }
    if let Some(anchor) = comments.body_anchor(owner_start) {
        emit_leading_comments(emitter, comments, anchor, false);
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts, comments));
    if let Some(anchor) = comments.closer_anchor(owner_start) {
        emit_leading_comments(emitter, comments, anchor, false);
    }
    emitter.write_current_indent();
    emitter.write(&format!("end {kind};"));
    emit_trailing_comments(emitter, comments, owner_start);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

/// Terminates a routine signature while preserving its trailing comments.
pub(super) fn finish_routine_header_line(
    emitter: &mut Emitter,
    comments: &CommentMap,
    owner_start: usize,
) {
    emitter.write(";");
    if let Some(anchor) = comments.header_anchor(owner_start) {
        emit_trailing_comments(emitter, comments, anchor);
    }
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

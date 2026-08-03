//! Individual declaration emission (const, var, type, routines).

use fpas_parser::{
    ConstDef, Decl, EnumMember, EnumType, FieldDef, FuncBody, FunctionDecl, ProcedureDecl,
    RecordEvent, RecordMethod, RecordProperty, RecordType, TypeBody, TypeDef, VarDef, Visibility,
};

use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::super::stmt::emit_stmts_in_block;
use super::super::types::{emit_formal_params_in_parens, emit_type_expr, format_type_params};

pub(crate) fn emit_decl(emitter: &mut Emitter, decl: &Decl, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, crate::span::decl_span(decl), true);
    match decl {
        Decl::Const(def) => emit_const_def(emitter, def, false, comments),
        Decl::Var(def) => emit_var_def(emitter, "var", def, false, comments),
        Decl::MutableVar(def) => emit_var_def(emitter, "mutable var", def, false, comments),
        Decl::TypeDef(def) => emit_type_def(emitter, def, comments, false),
        Decl::Function(function) => emit_function_decl(emitter, function, comments),
        Decl::Procedure(procedure) => emit_procedure_decl(emitter, procedure, comments),
    }
}

fn emit_visibility(emitter: &mut Emitter, visibility: Visibility) {
    if visibility == Visibility::Public {
        emitter.write("public ");
    }
}

pub(super) fn emit_const_def(
    emitter: &mut Emitter,
    def: &ConstDef,
    in_const_block: bool,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    if !in_const_block {
        emit_visibility(emitter, def.visibility);
    }
    if !in_const_block {
        emitter.write("const ");
    }
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0, comments);
    finish_decl_line(emitter, comments, def.span.offset);
}

pub(super) fn emit_var_def(
    emitter: &mut Emitter,
    keyword: &str,
    def: &VarDef,
    in_var_block: bool,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    if !in_var_block {
        emit_visibility(emitter, def.visibility);
        emitter.write(keyword);
        emitter.write(" ");
    }
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0, comments);
    finish_decl_line(emitter, comments, def.span.offset);
}

pub(super) fn emit_type_def(
    emitter: &mut Emitter,
    def: &TypeDef,
    comments: &CommentMap,
    in_type_block: bool,
) {
    emitter.write_current_indent();
    if !in_type_block {
        emit_visibility(emitter, def.visibility);
    }
    if !in_type_block {
        emitter.write("type ");
    }
    emitter.write(&def.name);
    emitter.write(" = ");
    emit_type_body(emitter, &def.body, comments);
    finish_decl_line(emitter, comments, def.span.offset);
}

fn emit_type_body(emitter: &mut Emitter, body: &TypeBody, comments: &CommentMap) {
    match body {
        TypeBody::Alias(type_expr) => emit_type_expr(emitter, type_expr),
        TypeBody::Record(record) => emit_record_type(emitter, record, comments),
        TypeBody::Enum(enum_type) => emit_enum_type(emitter, enum_type, comments),
    }
}

fn emit_record_type(emitter: &mut Emitter, record: &RecordType, comments: &CommentMap) {
    emitter.write("record\n");
    emitter.with_indent(|inner| {
        for field in &record.fields {
            emit_field_def(inner, field, comments);
        }
        if !record.fields.is_empty() && !record.methods.is_empty() {
            inner.write("\n");
        }
        for (index, method) in record.methods.iter().enumerate() {
            if index > 0 {
                inner.write("\n");
            }
            emit_record_method(inner, method, comments);
        }
        let need_property_gap = (!record.fields.is_empty() || !record.methods.is_empty())
            && !record.properties.is_empty();
        if need_property_gap {
            inner.write("\n");
        }
        for property in &record.properties {
            emit_record_property(inner, property, comments);
        }
        let need_event_gap = (!record.fields.is_empty()
            || !record.methods.is_empty()
            || !record.properties.is_empty())
            && !record.events.is_empty();
        if need_event_gap {
            inner.write("\n");
        }
        for event in &record.events {
            emit_record_event(inner, event, comments);
        }
    });
    emitter.write_current_indent();
    emitter.write("end");
}

fn emit_field_def(emitter: &mut Emitter, field: &FieldDef, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, field.span.offset, false);
    emitter.write_current_indent();
    emit_visibility(emitter, field.visibility);
    emitter.write(&field.name);
    emitter.write(": ");
    emit_type_expr(emitter, &field.type_expr);
    if let Some(default_value) = &field.default_value {
        emitter.write(" := ");
        emit_expr(emitter, default_value, 0, comments);
    }
    finish_decl_line(emitter, comments, field.span.offset);
}

fn emit_record_method(emitter: &mut Emitter, method: &RecordMethod, comments: &CommentMap) {
    match method {
        RecordMethod::Function(function) => {
            emit_leading_comments(emitter, comments, function.span.offset, true);
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
            emit_func_body(emitter, function.span.offset, &function.body, comments);
        }
        RecordMethod::StaticFunction(function) => {
            emit_leading_comments(emitter, comments, function.span.offset, true);
            emitter.write_current_indent();
            emit_visibility(emitter, function.visibility);
            emitter.write("static ");
            emit_function_header(
                emitter,
                &function.name,
                &function.type_params,
                &function.params,
            );
            emitter.write(": ");
            emit_type_expr(emitter, &function.return_type);
            finish_routine_header_line(emitter, comments, function.span.offset);
            emit_func_body(emitter, function.span.offset, &function.body, comments);
        }
        RecordMethod::StaticProcedure(procedure) => {
            emit_leading_comments(emitter, comments, procedure.span.offset, true);
            emitter.write_current_indent();
            emit_visibility(emitter, procedure.visibility);
            emitter.write("static ");
            emit_procedure_header(
                emitter,
                &procedure.name,
                &procedure.type_params,
                &procedure.params,
            );
            finish_routine_header_line(emitter, comments, procedure.span.offset);
            emit_func_body(emitter, procedure.span.offset, &procedure.body, comments);
        }
        RecordMethod::Procedure(procedure) => {
            emit_leading_comments(emitter, comments, procedure.span.offset, true);
            emitter.write_current_indent();
            emit_visibility(emitter, procedure.visibility);
            emit_procedure_header(
                emitter,
                &procedure.name,
                &procedure.type_params,
                &procedure.params,
            );
            finish_routine_header_line(emitter, comments, procedure.span.offset);
            emit_func_body(emitter, procedure.span.offset, &procedure.body, comments);
        }
    }
}

fn emit_record_property(emitter: &mut Emitter, property: &RecordProperty, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, property.span.offset, false);
    emitter.write_current_indent();
    emit_visibility(emitter, property.visibility);
    emitter.write("property ");
    emitter.write(&property.name);
    emitter.write(": ");
    emit_type_expr(emitter, &property.type_expr);
    if let Some(getter) = &property.read {
        emitter.write(" read ");
        emitter.write(getter);
    }
    if let Some(setter) = &property.write {
        emitter.write(" write ");
        emitter.write(setter);
    }
    finish_decl_line(emitter, comments, property.span.offset);
}

fn emit_record_event(emitter: &mut Emitter, event: &RecordEvent, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, event.span.offset, false);
    emitter.write_current_indent();
    emit_visibility(emitter, event.visibility);
    emitter.write("event ");
    emitter.write(&event.name);
    emitter.write(": ");
    emit_type_expr(emitter, &event.type_expr);
    emitter.write(" read ");
    emitter.write(&event.read);
    emitter.write(" write ");
    emitter.write(&event.write);
    finish_decl_line(emitter, comments, event.span.offset);
}

fn emit_enum_type(emitter: &mut Emitter, enum_type: &EnumType, comments: &CommentMap) {
    emitter.write("enum\n");
    emitter.with_indent(|inner| {
        for member in &enum_type.members {
            emit_enum_member(inner, member, comments);
        }
    });
    emitter.write_current_indent();
    emitter.write("end");
}

fn emit_enum_member(emitter: &mut Emitter, member: &EnumMember, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, member.span.offset, false);
    emitter.write_current_indent();
    emitter.write(&member.name);
    if !member.fields.is_empty() {
        emitter.write("(");
        for (index, field) in member.fields.iter().enumerate() {
            if index > 0 {
                emitter.write("; ");
            }
            emitter.write(&field.name);
            emitter.write(": ");
            emit_type_expr(emitter, &field.type_expr);
        }
        emitter.write(")");
    } else if let Some(value) = member.value {
        emitter.write(" = ");
        emitter.write(&value.to_string());
    }
    finish_decl_line(emitter, comments, member.span.offset);
}

fn emit_function_decl(emitter: &mut Emitter, function: &FunctionDecl, comments: &CommentMap) {
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
    emit_func_body(emitter, function.span.offset, &function.body, comments);
}

fn emit_procedure_decl(emitter: &mut Emitter, procedure: &ProcedureDecl, comments: &CommentMap) {
    emitter.write_current_indent();
    emit_visibility(emitter, procedure.visibility);
    emit_procedure_header(
        emitter,
        &procedure.name,
        &procedure.type_params,
        &procedure.params,
    );
    finish_routine_header_line(emitter, comments, procedure.span.offset);
    emit_func_body(emitter, procedure.span.offset, &procedure.body, comments);
}

fn emit_function_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("function {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

fn emit_procedure_header(
    emitter: &mut Emitter,
    name: &str,
    type_params: &[fpas_parser::TypeParam],
    params: &[fpas_parser::FormalParam],
) {
    let open = format!("procedure {name}{}(", format_type_params(type_params));
    emit_formal_params_in_parens(emitter, &open, params, "");
}

fn emit_func_body(
    emitter: &mut Emitter,
    owner_start: usize,
    body: &FuncBody,
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
    emitter.write_current_indent();
    emitter.write("end;");
    emit_trailing_comments(emitter, comments, owner_start);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

fn finish_routine_header_line(emitter: &mut Emitter, comments: &CommentMap, owner_start: usize) {
    emitter.write(";");
    if let Some(anchor) = comments.header_anchor(owner_start) {
        emit_trailing_comments(emitter, comments, anchor);
    }
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

fn finish_decl_line(emitter: &mut Emitter, comments: &CommentMap, anchor_start: usize) {
    emitter.write(";");
    emit_trailing_comments(emitter, comments, anchor_start);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

//! Individual declaration emission (const, var, type, routines).

use fpas_parser::{
    ConstDef, Decl, EnumMember, EnumType, FieldDef, FuncBody, FunctionDecl, ProcedureDecl,
    RecordMethod, RecordProperty, RecordType, TypeBody, TypeDef, VarDef, Visibility,
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
        Decl::Var(def) => emit_var_def(emitter, "var", def, comments),
        Decl::MutableVar(def) => emit_var_def(emitter, "mutable var", def, comments),
        Decl::TypeDef(def) => emit_type_def(emitter, def, comments, false),
        Decl::Function(function) => emit_function_decl(emitter, function, comments),
        Decl::Procedure(procedure) => emit_procedure_decl(emitter, procedure, comments),
    }
}

fn emit_visibility(emitter: &mut Emitter, visibility: Visibility) {
    if visibility == Visibility::Private {
        emitter.write("private ");
    }
}

pub(super) fn emit_const_def(
    emitter: &mut Emitter,
    def: &ConstDef,
    in_const_block: bool,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    emit_visibility(emitter, def.visibility);
    if !in_const_block {
        emitter.write("const ");
    }
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0);
    finish_decl_line(emitter, comments, def.span.offset);
}

pub(super) fn emit_var_def(
    emitter: &mut Emitter,
    keyword: &str,
    def: &VarDef,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    emit_visibility(emitter, def.visibility);
    emitter.write(keyword);
    emitter.write(" ");
    emitter.write(&def.name);
    emitter.write(": ");
    emit_type_expr(emitter, &def.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &def.value, 0);
    finish_decl_line(emitter, comments, def.span.offset);
}

pub(super) fn emit_type_def(
    emitter: &mut Emitter,
    def: &TypeDef,
    comments: &CommentMap,
    in_type_block: bool,
) {
    emitter.write_current_indent();
    emit_visibility(emitter, def.visibility);
    if !in_type_block {
        emitter.write("type ");
    }
    emitter.write(&def.name);
    emitter.write(" = ");
    emit_type_body(emitter, &def.body, comments);
    emitter.write(";");
    emit_trailing_comments(emitter, comments, def.span.offset);
    emitter.write("\n");
}

fn emit_type_body(emitter: &mut Emitter, body: &TypeBody, comments: &CommentMap) {
    match body {
        TypeBody::Alias(type_expr) => emit_type_expr(emitter, type_expr),
        TypeBody::Record(record) => emit_record_type(emitter, record, comments),
        TypeBody::Enum(enum_type) => emit_enum_type(emitter, enum_type),
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
    });
    emitter.write_current_indent();
    emitter.write("end");
}

fn emit_field_def(emitter: &mut Emitter, field: &FieldDef, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, field.span.offset, false);
    emitter.write_current_indent();
    emitter.write(&field.name);
    emitter.write(": ");
    emit_type_expr(emitter, &field.type_expr);
    if let Some(default_value) = &field.default_value {
        emitter.write(" := ");
        emit_expr(emitter, default_value, 0);
    }
    emitter.write(";\n");
}

fn emit_record_method(emitter: &mut Emitter, method: &RecordMethod, comments: &CommentMap) {
    match method {
        RecordMethod::Function(function) => {
            emit_leading_comments(emitter, comments, function.span.offset, true);
            emitter.write_current_indent();
            emit_function_header(
                emitter,
                &function.name,
                &function.type_params,
                &function.params,
            );
            emitter.write(": ");
            emit_type_expr(emitter, &function.return_type);
            emitter.write(";\n");
            emit_func_body(emitter, &function.body, comments);
        }
        RecordMethod::StaticFunction(function) => {
            emit_leading_comments(emitter, comments, function.span.offset, true);
            emitter.write_current_indent();
            emitter.write("static ");
            emit_function_header(
                emitter,
                &function.name,
                &function.type_params,
                &function.params,
            );
            emitter.write(": ");
            emit_type_expr(emitter, &function.return_type);
            emitter.write(";\n");
            emit_func_body(emitter, &function.body, comments);
        }
        RecordMethod::Procedure(procedure) => {
            emit_leading_comments(emitter, comments, procedure.span.offset, true);
            emitter.write_current_indent();
            emit_procedure_header(
                emitter,
                &procedure.name,
                &procedure.type_params,
                &procedure.params,
            );
            emitter.write(";\n");
            emit_func_body(emitter, &procedure.body, comments);
        }
    }
}

fn emit_record_property(emitter: &mut Emitter, property: &RecordProperty, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, property.span.offset, false);
    emitter.write_current_indent();
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
    emitter.write(";\n");
}

fn emit_enum_type(emitter: &mut Emitter, enum_type: &EnumType) {
    emitter.write("enum\n");
    emitter.with_indent(|inner| {
        for member in &enum_type.members {
            emit_enum_member(inner, member);
        }
    });
    emitter.write_current_indent();
    emitter.write("end");
}

fn emit_enum_member(emitter: &mut Emitter, member: &EnumMember) {
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
    emitter.write(";\n");
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
    emitter.write(";\n");
    emit_func_body(emitter, &function.body, comments);
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
    emitter.write(";\n");
    emit_func_body(emitter, &procedure.body, comments);
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

fn emit_func_body(emitter: &mut Emitter, body: &FuncBody, comments: &CommentMap) {
    let FuncBody::Block { nested, stmts } = body;
    for decl in nested {
        emit_decl(emitter, decl, comments);
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts, comments));
    emitter.write_current_indent();
    emitter.write("end;\n");
}

fn finish_decl_line(emitter: &mut Emitter, comments: &CommentMap, anchor_start: usize) {
    emitter.write(";");
    emit_trailing_comments(emitter, comments, anchor_start);
    emitter.write_line_end();
}

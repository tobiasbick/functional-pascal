use super::expressions::apply_expr_source_id;
use super::statements::apply_stmt_source_id;
use super::support::apply_span;
use super::types::{apply_formal_param_source_id, apply_type_expr_source_id};

use fpas_parser::{
    Decl, EnumMember, EnumMemberField, FieldDef, FuncBody, FunctionDecl, ProcedureDecl,
    RecordMethod, RecordType, TypeBody, TypeDef, VarDef,
};

pub(super) fn apply_decl_source_id(decl: &mut Decl, source_id: u32) {
    match decl {
        Decl::Const(const_def) => {
            apply_type_expr_source_id(&mut const_def.type_expr, source_id);
            apply_expr_source_id(&mut const_def.value, source_id);
            apply_span(&mut const_def.span, source_id);
        }
        Decl::Var(var_def) | Decl::MutableVar(var_def) => {
            apply_var_def_source_id(var_def, source_id);
        }
        Decl::TypeDef(type_def) => {
            apply_type_def_source_id(type_def, source_id);
        }
        Decl::Function(function) => apply_function_source_id(function, source_id),
        Decl::Procedure(procedure) => apply_procedure_source_id(procedure, source_id),
    }
}

pub(super) fn apply_var_def_source_id(var_def: &mut VarDef, source_id: u32) {
    apply_type_expr_source_id(&mut var_def.type_expr, source_id);
    apply_expr_source_id(&mut var_def.value, source_id);
    apply_span(&mut var_def.span, source_id);
}

fn apply_type_def_source_id(type_def: &mut TypeDef, source_id: u32) {
    match &mut type_def.body {
        TypeBody::Record(record) => apply_record_type_source_id(record, source_id),
        TypeBody::Enum(enum_type) => {
            for member in &mut enum_type.members {
                apply_enum_member_source_id(member, source_id);
            }
            apply_span(&mut enum_type.span, source_id);
        }
        TypeBody::Alias(type_expr) => apply_type_expr_source_id(type_expr, source_id),
    }
    apply_span(&mut type_def.span, source_id);
}

fn apply_record_type_source_id(record: &mut RecordType, source_id: u32) {
    for field in &mut record.fields {
        apply_field_def_source_id(field, source_id);
    }
    for method in &mut record.methods {
        match method {
            RecordMethod::Function(function) | RecordMethod::StaticFunction(function) => {
                apply_function_source_id(function, source_id)
            }
            RecordMethod::Procedure(procedure) => apply_procedure_source_id(procedure, source_id),
        }
    }
    apply_span(&mut record.span, source_id);
}

fn apply_field_def_source_id(field: &mut FieldDef, source_id: u32) {
    apply_type_expr_source_id(&mut field.type_expr, source_id);
    if let Some(default_value) = &mut field.default_value {
        apply_expr_source_id(default_value, source_id);
    }
    apply_span(&mut field.span, source_id);
}

fn apply_enum_member_source_id(member: &mut EnumMember, source_id: u32) {
    for field in &mut member.fields {
        apply_enum_member_field_source_id(field, source_id);
    }
    apply_span(&mut member.span, source_id);
}

fn apply_enum_member_field_source_id(field: &mut EnumMemberField, source_id: u32) {
    apply_type_expr_source_id(&mut field.type_expr, source_id);
    apply_span(&mut field.span, source_id);
}

fn apply_function_source_id(function: &mut FunctionDecl, source_id: u32) {
    for param in &mut function.params {
        apply_formal_param_source_id(param, source_id);
    }
    apply_type_expr_source_id(&mut function.return_type, source_id);
    apply_func_body_source_id(&mut function.body, source_id);
    apply_span(&mut function.span, source_id);
}

fn apply_procedure_source_id(procedure: &mut ProcedureDecl, source_id: u32) {
    for param in &mut procedure.params {
        apply_formal_param_source_id(param, source_id);
    }
    apply_func_body_source_id(&mut procedure.body, source_id);
    apply_span(&mut procedure.span, source_id);
}

pub(super) fn apply_func_body_source_id(body: &mut FuncBody, source_id: u32) {
    let FuncBody::Block { nested, stmts } = body;
    for decl in nested {
        apply_decl_source_id(decl, source_id);
    }
    for stmt in stmts {
        apply_stmt_source_id(stmt, source_id);
    }
}

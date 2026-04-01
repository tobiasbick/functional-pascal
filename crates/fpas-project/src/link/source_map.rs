//! Source-ID assignment for linked project programs.
//!
//! **Documentation:** `docs/pascal/09-units.md`, `docs/pascal/10-projects.md`

use fpas_lexer::Span;
use fpas_parser::{
    CaseArm, CaseLabel, Decl, Designator, DesignatorPart, EnumMember, EnumMemberField, Expr,
    FieldDef, FieldInit, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, Program, QualifiedId,
    RecordMethod, RecordType, Stmt, TypeBody, TypeDef, TypeExpr, Unit, VarDef,
};

pub(super) fn apply_program_source_id(program: &mut Program, source_id: u32) {
    apply_span(&mut program.name_span, source_id);
    for uses in &mut program.uses {
        apply_qualified_id_source_id(uses, source_id);
    }
    for declaration in &mut program.declarations {
        apply_decl_source_id(declaration, source_id);
    }
    for stmt in &mut program.body {
        apply_stmt_source_id(stmt, source_id);
    }
    apply_span(&mut program.span, source_id);
}

pub(super) fn apply_unit_source_id(unit: &mut Unit, source_id: u32) {
    apply_qualified_id_source_id(&mut unit.name, source_id);
    for uses in &mut unit.uses {
        apply_qualified_id_source_id(uses, source_id);
    }
    for declaration in &mut unit.declarations {
        apply_decl_source_id(declaration, source_id);
    }
    apply_span(&mut unit.span, source_id);
}

fn apply_decl_source_id(decl: &mut Decl, source_id: u32) {
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

fn apply_var_def_source_id(var_def: &mut VarDef, source_id: u32) {
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
            RecordMethod::Function(function) => apply_function_source_id(function, source_id),
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

fn apply_formal_param_source_id(param: &mut FormalParam, source_id: u32) {
    apply_type_expr_source_id(&mut param.type_expr, source_id);
    apply_span(&mut param.span, source_id);
}

fn apply_func_body_source_id(body: &mut FuncBody, source_id: u32) {
    let FuncBody::Block { nested, stmts } = body;
    for decl in nested {
        apply_decl_source_id(decl, source_id);
    }
    for stmt in stmts {
        apply_stmt_source_id(stmt, source_id);
    }
}

fn apply_stmt_source_id(stmt: &mut Stmt, source_id: u32) {
    match stmt {
        Stmt::Block(stmts, span) => {
            for stmt in stmts {
                apply_stmt_source_id(stmt, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Var(var_def) | Stmt::MutableVar(var_def) => {
            apply_var_def_source_id(var_def, source_id)
        }
        Stmt::Assign {
            target,
            value,
            span,
        } => {
            apply_designator_source_id(target, source_id);
            apply_expr_source_id(value, source_id);
            apply_span(span, source_id);
        }
        Stmt::Return(expr, span) => {
            if let Some(expr) = expr {
                apply_expr_source_id(expr, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Panic(expr, span) => {
            apply_expr_source_id(expr, source_id);
            apply_span(span, source_id);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            apply_expr_source_id(condition, source_id);
            apply_stmt_source_id(then_branch, source_id);
            if let Some(else_branch) = else_branch {
                apply_stmt_source_id(else_branch, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Case {
            expr,
            arms,
            else_body,
            span,
        } => {
            apply_expr_source_id(expr, source_id);
            for arm in arms {
                apply_case_arm_source_id(arm, source_id);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    apply_stmt_source_id(stmt, source_id);
                }
            }
            apply_span(span, source_id);
        }
        Stmt::For {
            var_type,
            start,
            direction: _,
            end,
            body,
            span,
            ..
        } => {
            apply_type_expr_source_id(var_type, source_id);
            apply_expr_source_id(start, source_id);
            apply_expr_source_id(end, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::ForIn {
            var_type,
            iterable,
            body,
            span,
            ..
        } => {
            apply_type_expr_source_id(var_type, source_id);
            apply_expr_source_id(iterable, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            apply_expr_source_id(condition, source_id);
            apply_stmt_source_id(body, source_id);
            apply_span(span, source_id);
        }
        Stmt::Repeat {
            body,
            condition,
            span,
        } => {
            for stmt in body {
                apply_stmt_source_id(stmt, source_id);
            }
            apply_expr_source_id(condition, source_id);
            apply_span(span, source_id);
        }
        Stmt::Break(span) | Stmt::Continue(span) => apply_span(span, source_id),
        Stmt::Call {
            designator,
            args,
            span,
        } => {
            apply_designator_source_id(designator, source_id);
            for arg in args {
                apply_expr_source_id(arg, source_id);
            }
            apply_span(span, source_id);
        }
        Stmt::Go { expr, span } => {
            apply_expr_source_id(expr, source_id);
            apply_span(span, source_id);
        }
    }
}

fn apply_case_arm_source_id(arm: &mut CaseArm, source_id: u32) {
    for label in &mut arm.labels {
        apply_case_label_source_id(label, source_id);
    }
    if let Some(guard) = &mut arm.guard {
        apply_expr_source_id(guard, source_id);
    }
    apply_stmt_source_id(&mut arm.body, source_id);
    apply_span(&mut arm.span, source_id);
}

fn apply_case_label_source_id(label: &mut CaseLabel, source_id: u32) {
    match label {
        CaseLabel::Value { start, end, span } => {
            apply_expr_source_id(start, source_id);
            if let Some(end) = end {
                apply_expr_source_id(end, source_id);
            }
            apply_span(span, source_id);
        }
        CaseLabel::Destructure { span, .. } => apply_span(span, source_id),
    }
}

fn apply_expr_source_id(expr: &mut Expr, source_id: u32) {
    match expr {
        Expr::Integer(_, span)
        | Expr::Real(_, span)
        | Expr::Str(_, span)
        | Expr::Bool(_, span)
        | Expr::Paren(_, span)
        | Expr::ArrayLiteral(_, span)
        | Expr::DictLiteral(_, span)
        | Expr::ResultOk(_, span)
        | Expr::ResultError(_, span)
        | Expr::OptionSome(_, span)
        | Expr::OptionNone(span)
        | Expr::Try(_, span)
        | Expr::Go(_, span)
        | Expr::Error(span) => {
            apply_span(span, source_id);
        }
        Expr::Designator(designator) => apply_designator_source_id(designator, source_id),
        Expr::Call {
            designator,
            args,
            span,
        } => {
            apply_designator_source_id(designator, source_id);
            for arg in args {
                apply_expr_source_id(arg, source_id);
            }
            apply_span(span, source_id);
        }
        Expr::UnaryOp { operand, span, .. } => {
            apply_expr_source_id(operand, source_id);
            apply_span(span, source_id);
        }
        Expr::BinaryOp {
            left, right, span, ..
        } => {
            apply_expr_source_id(left, source_id);
            apply_expr_source_id(right, source_id);
            apply_span(span, source_id);
        }
        Expr::RecordLiteral { fields, span } => {
            for field in fields {
                apply_field_init_source_id(field, source_id);
            }
            apply_span(span, source_id);
        }
        Expr::RecordUpdate { base, fields, span } => {
            apply_expr_source_id(base, source_id);
            for field in fields {
                apply_field_init_source_id(field, source_id);
            }
            apply_span(span, source_id);
        }
    }

    match expr {
        Expr::Paren(inner, _)
        | Expr::ResultOk(inner, _)
        | Expr::ResultError(inner, _)
        | Expr::OptionSome(inner, _)
        | Expr::Try(inner, _)
        | Expr::Go(inner, _) => {
            apply_expr_source_id(inner, source_id);
        }
        Expr::ArrayLiteral(elements, _) => {
            for element in elements {
                apply_expr_source_id(element, source_id);
            }
        }
        Expr::DictLiteral(entries, _) => {
            for (key, value) in entries {
                apply_expr_source_id(key, source_id);
                apply_expr_source_id(value, source_id);
            }
        }
        _ => {}
    }
}

fn apply_field_init_source_id(field: &mut FieldInit, source_id: u32) {
    apply_expr_source_id(&mut field.value, source_id);
    apply_span(&mut field.span, source_id);
}

fn apply_designator_source_id(designator: &mut Designator, source_id: u32) {
    for part in &mut designator.parts {
        match part {
            DesignatorPart::Ident(_, span) => apply_span(span, source_id),
            DesignatorPart::Index(expr, span) => {
                apply_expr_source_id(expr, source_id);
                apply_span(span, source_id);
            }
        }
    }
    apply_span(&mut designator.span, source_id);
}

fn apply_type_expr_source_id(type_expr: &mut TypeExpr, source_id: u32) {
    match type_expr {
        TypeExpr::Named { id, span } => {
            apply_qualified_id_source_id(id, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::Array(inner, span)
        | TypeExpr::Option {
            inner_type: inner,
            span,
        } => {
            apply_type_expr_source_id(inner, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::FunctionType {
            params,
            return_type,
            span,
        } => {
            for param in params {
                apply_formal_param_source_id(param, source_id);
            }
            apply_type_expr_source_id(return_type, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::ProcedureType { params, span } => {
            for param in params {
                apply_formal_param_source_id(param, source_id);
            }
            apply_span(span, source_id);
        }
        TypeExpr::Result {
            ok_type,
            err_type,
            span,
        } => {
            apply_type_expr_source_id(ok_type, source_id);
            apply_type_expr_source_id(err_type, source_id);
            apply_span(span, source_id);
        }
        TypeExpr::Dict {
            key_type,
            value_type,
            span,
        } => {
            apply_type_expr_source_id(key_type, source_id);
            apply_type_expr_source_id(value_type, source_id);
            apply_span(span, source_id);
        }
    }
}

fn apply_qualified_id_source_id(id: &mut QualifiedId, source_id: u32) {
    apply_span(&mut id.span, source_id);
}

fn apply_span(span: &mut Span, source_id: u32) {
    span.source_id = source_id;
}

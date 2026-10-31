//! Public declaration projection using the ordinary formatter's emitters.

use fpas_parser::{Decl, RecordMethod, TypeBody, Unit, Visibility};

use super::super::{
    Emitter,
    program::emit_optional_uses,
    types::{emit_qualified_id, emit_type_expr},
};
use super::{item, routines};
use crate::comments::CommentMap;

/// Emits the current unit's imports and public source declarations in source order.
pub(crate) fn format_public_signatures(unit: &Unit, comments: &CommentMap) -> String {
    let mut emitter = Emitter::with_comments(comments);
    let plain = CommentMap::default();
    emitter.write("unit ");
    emit_qualified_id(&mut emitter, &unit.name);
    emitter.write(";\n");
    emitter.blank_line();
    emit_optional_uses(&mut emitter, &unit.uses, &plain);
    for declaration in unit
        .declarations
        .iter()
        .filter(|decl| decl.visibility() == Visibility::Public)
    {
        match declaration {
            Decl::Function(value) => function(&mut emitter, value, "", &plain),
            Decl::Procedure(value) => procedure(&mut emitter, value, "", &plain),
            Decl::Const(value) => binding(&mut emitter, "const", &value.name, &value.type_expr),
            Decl::Var(value) => binding(&mut emitter, "var", &value.name, &value.type_expr),
            Decl::TypeDef(value) => {
                if let TypeBody::Record(record) = &value.body {
                    item::emit_type_heading(&mut emitter, value);
                    emitter.write("record\n");
                    emitter.with_indent(|inner| {
                        item::emit_record_members(
                            inner,
                            record
                                .fields
                                .iter()
                                .filter(|field| field.visibility == Visibility::Public),
                            record
                                .methods
                                .iter()
                                .filter(|method| method.visibility() == Visibility::Public),
                            &plain,
                            public_method,
                        );
                    });
                    emitter.writeln("end record;");
                } else {
                    item::emit_decl(&mut emitter, declaration, &plain);
                }
            }
        }
        emitter.blank_line();
    }
    emitter.writeln("end unit;");
    emitter.finish()
}

fn public_method(emitter: &mut Emitter, method: &RecordMethod, comments: &CommentMap) {
    match method {
        RecordMethod::Function(value) => function(emitter, value, "", comments),
        RecordMethod::StaticFunction(value) => function(emitter, value, "static ", comments),
        RecordMethod::Procedure(value) => procedure(
            emitter,
            value,
            if value.is_release { "release " } else { "" },
            comments,
        ),
        RecordMethod::StaticProcedure(value) => procedure(emitter, value, "static ", comments),
    }
}

fn binding(emitter: &mut Emitter, keyword: &str, name: &str, ty: &fpas_parser::TypeExpr) {
    emitter.write(&format!("public {keyword} {name}: "));
    emit_type_expr(emitter, ty);
    emitter.write(";\n");
}

fn function(
    emitter: &mut Emitter,
    value: &fpas_parser::FunctionDecl,
    prefix: &str,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    item::emit_visibility(emitter, value.visibility);
    emitter.write(prefix);
    routines::emit_function_signature(emitter, value, comments);
}

fn procedure(
    emitter: &mut Emitter,
    value: &fpas_parser::ProcedureDecl,
    prefix: &str,
    comments: &CommentMap,
) {
    emitter.write_current_indent();
    item::emit_visibility(emitter, value.visibility);
    emitter.write(prefix);
    routines::emit_procedure_signature(emitter, value, comments);
}

//! Validation of type visibility across public unit-interface signatures.

use std::collections::HashSet;

use fpas_diagnostics::codes::SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE;
use fpas_lexer::Span;
use fpas_parser::{Decl, RecordMethod, TypeBody, TypeExpr, Unit, Visibility};

use crate::SemaError;
use crate::error::sema_error;
use crate::scope::canonical_symbol_name;

/// Reject public declarations whose persisted signature refers to a private type.
///
/// **Documentation:** `docs/pascal/program-structure/visibility.md`
pub(super) fn validate(unit: &Unit) -> Vec<SemaError> {
    let private_types = PrivateTypes {
        names: unit
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Decl::TypeDef(definition) if definition.visibility == Visibility::Private => {
                    Some(canonical_symbol_name(&definition.name))
                }
                _ => None,
            })
            .collect(),
        unit_name: &unit.name.parts,
    };

    unit.declarations
        .iter()
        .filter(|declaration| declaration.visibility() == Visibility::Public)
        .filter_map(|declaration| {
            let private_type = private_type_in_declaration(declaration, &private_types)?;
            let declaration_name = super::export::declaration_name(declaration);
            Some(sema_error(
                SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE,
                format!(
                    "public declaration `{declaration_name}` uses private type `{}` in its signature",
                    private_type.name
                ),
                format!(
                    "Make type `{}` public or stop exporting `{declaration_name}`.",
                    private_type.name
                ),
                private_type.span,
            ))
        })
        .collect()
}

#[derive(Clone, Copy)]
struct PrivateTypeReference<'a> {
    name: &'a str,
    span: Span,
}

struct PrivateTypes<'a> {
    names: HashSet<String>,
    unit_name: &'a [String],
}

impl PrivateTypes<'_> {
    fn contains(&self, id: &fpas_parser::QualifiedId) -> bool {
        let Some(name) = id.parts.last() else {
            return false;
        };
        if !self.names.contains(&canonical_symbol_name(name)) {
            return false;
        }
        id.parts.len() == 1
            || id.parts.len() == self.unit_name.len() + 1
                && id.parts[..id.parts.len() - 1]
                    .iter()
                    .zip(self.unit_name)
                    .all(|(left, right)| left.eq_ignore_ascii_case(right))
    }
}

fn private_type_in_declaration<'a>(
    declaration: &'a Decl,
    private_types: &PrivateTypes<'_>,
) -> Option<PrivateTypeReference<'a>> {
    match declaration {
        Decl::Const(definition) => private_type_in(&definition.type_expr, private_types),
        Decl::Var(definition) | Decl::MutableVar(definition) => {
            private_type_in(&definition.type_expr, private_types)
        }
        Decl::Function(function) => private_type_in_function(function, private_types),
        Decl::Procedure(procedure) => private_type_in_parameters(&procedure.params, private_types),
        Decl::TypeDef(definition) => match &definition.body {
            TypeBody::Alias(ty) => private_type_in(ty, private_types),
            TypeBody::Enum(enumeration) => enumeration
                .members
                .iter()
                .flat_map(|variant| &variant.fields)
                .find_map(|field| private_type_in(&field.type_expr, private_types)),
            TypeBody::Record(record) => record
                .fields
                .iter()
                .find_map(|field| private_type_in(&field.type_expr, private_types))
                .or_else(|| {
                    record.methods.iter().find_map(|method| match method {
                        RecordMethod::Function(function)
                        | RecordMethod::StaticFunction(function) => {
                            private_type_in_function(function, private_types)
                        }
                        RecordMethod::Procedure(procedure)
                        | RecordMethod::StaticProcedure(procedure) => {
                            private_type_in_parameters(&procedure.params, private_types)
                        }
                    })
                })
                .or_else(|| {
                    record
                        .properties
                        .iter()
                        .find_map(|property| private_type_in(&property.type_expr, private_types))
                })
                .or_else(|| {
                    record
                        .events
                        .iter()
                        .find_map(|event| private_type_in(&event.type_expr, private_types))
                }),
        },
    }
}

fn private_type_in_function<'a>(
    function: &'a fpas_parser::FunctionDecl,
    private_types: &PrivateTypes<'_>,
) -> Option<PrivateTypeReference<'a>> {
    private_type_in_parameters(&function.params, private_types)
        .or_else(|| private_type_in(&function.return_type, private_types))
}

fn private_type_in_parameters<'a>(
    parameters: &'a [fpas_parser::FormalParam],
    private_types: &PrivateTypes<'_>,
) -> Option<PrivateTypeReference<'a>> {
    parameters
        .iter()
        .find_map(|parameter| private_type_in(&parameter.type_expr, private_types))
}

fn private_type_in<'a>(
    ty: &'a TypeExpr,
    private_types: &PrivateTypes<'_>,
) -> Option<PrivateTypeReference<'a>> {
    match ty {
        TypeExpr::Named { id, span } => {
            let name = id.parts.last()?;
            private_types
                .contains(id)
                .then_some(PrivateTypeReference { name, span: *span })
        }
        TypeExpr::Array(inner, _)
        | TypeExpr::Channel(inner, _)
        | TypeExpr::Option {
            inner_type: inner, ..
        } => private_type_in(inner, private_types),
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => private_type_in(key_type, private_types)
            .or_else(|| private_type_in(value_type, private_types)),
        TypeExpr::Result {
            ok_type, err_type, ..
        } => private_type_in(ok_type, private_types)
            .or_else(|| private_type_in(err_type, private_types)),
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => private_type_in_parameters(params, private_types)
            .or_else(|| private_type_in(return_type, private_types)),
        TypeExpr::ProcedureType { params, .. } => private_type_in_parameters(params, private_types),
    }
}

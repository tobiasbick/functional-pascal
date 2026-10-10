//! Generic references in recursive type cycles must forward parameters unchanged.
//! See `docs/pascal/language/types/generics.md`.

use crate::check::Checker;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{Decl, TypeBody, TypeDef, TypeExpr};
use std::collections::{HashMap, HashSet};

struct Reference<'a> {
    target: usize,
    arguments: Option<&'a [TypeExpr]>,
    span: fpas_lexer::Span,
}

impl Checker {
    /// Validate recursive generic applications before concrete construction checks.
    pub(super) fn validate_generic_recursion(&mut self, declarations: &[Decl]) {
        let definitions: Vec<_> = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Decl::TypeDef(definition) if self.has_collected_type(definition) => {
                    Some(definition)
                }
                _ => None,
            })
            .collect();
        if definitions
            .iter()
            .all(|definition| definition.type_params.is_empty())
        {
            return;
        }
        let names: HashMap<_, _> = definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| (definition.name.to_ascii_lowercase(), index))
            .collect();
        let graph: Vec<_> = definitions
            .iter()
            .map(|definition| references(definition, &names))
            .collect();
        for (source, edges) in graph.iter().enumerate() {
            for edge in edges {
                let Some(arguments) = edge.arguments else {
                    continue;
                };
                if definitions[edge.target].type_params.is_empty()
                    || !reaches(&graph, edge.target, source)
                {
                    continue;
                }
                let parameters = &definitions[source].type_params;
                let forwards = arguments.len() == parameters.len() && arguments.iter().zip(parameters).all(|(argument, parameter)| {
                    matches!(argument, TypeExpr::Named { id, .. } if id.parts.len() == 1 && id.parts[0].eq_ignore_ascii_case(&parameter.name))
                });
                if !forwards {
                    self.error_with_code(SEMA_TYPE_MISMATCH,
                        format!("Recursive reference to `{}` must forward the declaring type parameters unchanged and in declaration order", definitions[edge.target].name),
                        "Use `Node of T` or `Node of (K, V)` with the same parameters; recursive arguments cannot be reordered, replaced, or wrapped.", edge.span);
                    if let Some(symbol) = self.scopes.lookup_mut(&definitions[source].name) {
                        symbol.ty = crate::types::Ty::Error;
                    }
                }
            }
        }
    }
}

fn references<'a>(definition: &'a TypeDef, names: &HashMap<String, usize>) -> Vec<Reference<'a>> {
    let mut result = Vec::new();
    let mut pending = Vec::new();
    match &definition.body {
        TypeBody::Record(record) => {
            pending.extend(record.fields.iter().map(|field| &field.type_expr))
        }
        TypeBody::Enum(enumeration) => pending.extend(
            enumeration
                .members
                .iter()
                .flat_map(|member| member.fields.iter().map(|field| &field.type_expr)),
        ),
        TypeBody::Alias(ty) | TypeBody::Distinct(ty) => pending.push(ty),
    }
    while let Some(ty) = pending.pop() {
        match ty {
            TypeExpr::Named { id, span } | TypeExpr::Application { id, span, .. } => {
                let name = id.parts.join(".").to_ascii_lowercase();
                if definition
                    .type_params
                    .iter()
                    .any(|parameter| parameter.name.eq_ignore_ascii_case(&name))
                {
                    continue;
                }
                if let Some(&target) = names.get(&name) {
                    result.push(Reference {
                        target,
                        arguments: match ty {
                            TypeExpr::Application { arguments, .. } => Some(arguments),
                            _ => None,
                        },
                        span: *span,
                    });
                }
                if let TypeExpr::Application { arguments, .. } = ty {
                    pending.extend(arguments);
                }
            }
            TypeExpr::Array(inner, _)
            | TypeExpr::Channel(inner, _)
            | TypeExpr::Task(inner, _)
            | TypeExpr::Option {
                inner_type: inner, ..
            } => pending.push(inner),
            TypeExpr::Result {
                ok_type, err_type, ..
            } => pending.extend([ok_type.as_ref(), err_type.as_ref()]),
            TypeExpr::Dict {
                key_type,
                value_type,
                ..
            } => pending.extend([key_type.as_ref(), value_type.as_ref()]),
            TypeExpr::FunctionType {
                params,
                return_type,
                ..
            } => {
                pending.extend(params.iter().map(|parameter| &parameter.type_expr));
                pending.push(return_type);
            }
            TypeExpr::ProcedureType { params, .. } => {
                pending.extend(params.iter().map(|parameter| &parameter.type_expr))
            }
        }
    }
    result
}

fn reaches(graph: &[Vec<Reference<'_>>], start: usize, target: usize) -> bool {
    let mut pending = vec![start];
    let mut visited = HashSet::new();
    while let Some(node) = pending.pop() {
        if node == target {
            return true;
        }
        if visited.insert(node) {
            pending.extend(graph[node].iter().map(|edge| edge.target));
        }
    }
    false
}

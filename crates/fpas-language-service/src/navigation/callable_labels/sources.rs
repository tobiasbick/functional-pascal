//! Written annotations, type aliases, field declarations and routine return signatures.

use super::{Step, Visited};
use crate::{
    DocumentSymbol, SymbolKind,
    navigation::{NavigationDocument, find_type, resolve},
};
use fpas_lexer::Token;
use fpas_parser::{CompilationUnit, Decl, FormalParam, TypeBody, TypeExpr};

/// Resolves a formal parameter at its original source position, including a `var` prefix.
pub(super) fn source_parameter(
    documents: &[NavigationDocument],
    owner: usize,
    parameter: &FormalParam,
    start: usize,
) -> Option<(usize, DocumentSymbol)> {
    let token = documents[owner].tokens.iter().find(|token| token.span.offset >= start + parameter.span.offset && token.span.offset < start + parameter.span.offset + parameter.span.length && matches!(&token.token, Token::Ident(name) if name.eq_ignore_ascii_case(&parameter.name)))?;
    let (_, symbol, _) = resolve(documents, owner, token.span.offset)?;
    (symbol.kind == SymbolKind::Parameter).then_some((owner, symbol))
}

/// Follows the selected component of a written type to its declared callable labels.
pub(super) fn type_parameter(
    documents: &[NavigationDocument],
    owner: usize,
    syntax: &TypeExpr,
    start: usize,
    name: &str,
    path: &[Step],
    visited: &mut Visited,
) -> Option<(usize, DocumentSymbol)> {
    match (syntax, path.split_first()) {
        (TypeExpr::FunctionType { return_type, .. }, Some((Step::Return, rest))) => {
            type_parameter(documents, owner, return_type, start, name, rest, visited)
        }
        (TypeExpr::FunctionType { params, .. } | TypeExpr::ProcedureType { params, .. }, None) => {
            source_parameter(
                documents,
                owner,
                params
                    .iter()
                    .find(|param| param.name.eq_ignore_ascii_case(name))?,
                start,
            )
        }
        (TypeExpr::Named { id, .. } | TypeExpr::Application { id, .. }, _) => {
            let (index, symbol) = find_type(documents, owner, owner, &id.parts.join("."))?;
            super::parameter_inner(documents, index, symbol, name, path, visited)
        }
        (
            TypeExpr::Array(inner, _)
            | TypeExpr::Dict {
                value_type: inner, ..
            },
            Some((Step::Element, rest)),
        )
        | (
            TypeExpr::Option {
                inner_type: inner, ..
            },
            Some((Step::Some, rest)),
        )
        | (TypeExpr::Result { ok_type: inner, .. }, Some((Step::Ok, rest)))
        | (
            TypeExpr::Result {
                err_type: inner, ..
            },
            Some((Step::Error, rest)),
        ) => type_parameter(documents, owner, inner, start, name, rest, visited),
        _ => None,
    }
}

/// Resolves aliases and declared aggregate fields in their source unit's namespace.
pub(super) fn declaration_parameter(
    documents: &[NavigationDocument],
    owner: usize,
    symbol: &DocumentSymbol,
    name: &str,
    path: &[Step],
    visited: &mut Visited,
) -> Option<(usize, DocumentSymbol)> {
    let declarations = match documents[owner].snapshot.compilation_unit() {
        CompilationUnit::Program(program) => &program.declarations,
        CompilationUnit::Unit(unit) => &unit.declarations,
    };
    let definition = declarations
        .iter()
        .find_map(|declaration| match declaration {
            Decl::TypeDef(definition) if definition.span.offset == symbol.full_span.offset() => {
                Some(definition)
            }
            _ => None,
        })?;
    match (&definition.body, path.split_first()) {
        (TypeBody::Alias(syntax), _) => {
            type_parameter(documents, owner, syntax, 0, name, path, visited)
        }
        (
            TypeBody::Record(record),
            Some((
                Step::Field {
                    variant: None,
                    name: Some(field),
                    ..
                },
                rest,
            )),
        ) => type_parameter(
            documents,
            owner,
            &record
                .fields
                .iter()
                .find(|item| {
                    item.name.eq_ignore_ascii_case(field)
                        && (owner == visited.target
                            || item.visibility == fpas_parser::Visibility::Public)
                })?
                .type_expr,
            0,
            name,
            rest,
            visited,
        ),
        (
            TypeBody::Enum(enumeration),
            Some((
                Step::Field {
                    variant: Some(variant),
                    name: field,
                    position,
                },
                rest,
            )),
        ) => {
            let variant = enumeration
                .members
                .iter()
                .find(|item| item.name.eq_ignore_ascii_case(variant))?;
            let field = if let Some(name) = field {
                variant
                    .fields
                    .iter()
                    .find(|field| field.name.eq_ignore_ascii_case(name))?
            } else {
                variant.fields.get(*position)?
            };
            type_parameter(documents, owner, &field.type_expr, 0, name, rest, visited)
        }
        _ => None,
    }
}

/// Uses a routine's written return type, including a callable nested in an aggregate.
pub(super) fn return_parameter(
    documents: &[NavigationDocument],
    owner: usize,
    symbol: &DocumentSymbol,
    name: &str,
    path: &[Step],
    visited: &mut Visited,
) -> Option<(usize, DocumentSymbol)> {
    let document = &documents[owner];
    let tokens = document
        .tokens
        .iter()
        .filter(|token| {
            token.span.offset >= symbol.full_span.offset()
                && token.span.offset < symbol.full_span.end()
        })
        .collect::<Vec<_>>();
    let open = tokens
        .iter()
        .position(|token| token.token == Token::LParen)?;
    let mut depth = 0usize;
    let close = (open..tokens.len()).find(|&index| match tokens[index].token {
        Token::LParen => {
            depth += 1;
            false
        }
        Token::RParen => {
            depth -= 1;
            depth == 0
        }
        _ => false,
    })?;
    if tokens.get(close + 1)?.token != Token::Colon {
        return None;
    }
    let start = tokens.get(close + 2)?.span.offset;
    let mut depth = 0usize;
    let end = (close + 2..tokens.len()).find(|&index| match tokens[index].token {
        Token::LParen => {
            depth += 1;
            false
        }
        Token::RParen => {
            depth = depth.saturating_sub(1);
            false
        }
        Token::Semicolon if depth == 0 => true,
        _ => false,
    })?;
    let (syntax, errors) = fpas_parser::parse_type_expression(
        document
            .snapshot
            .source()
            .get(start..tokens[end].span.offset)?
            .trim(),
    );
    if !errors.is_empty() {
        return None;
    }
    type_parameter(documents, owner, &syntax, start, name, path, visited)
}

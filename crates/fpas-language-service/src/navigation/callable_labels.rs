//! Source owners of receiving callable labels, independent of implementation names.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`

mod paths;
mod sources;

use super::{NavigationDocument, resolve};
use crate::{DocumentSymbol, SymbolKind};
use fpas_lexer::Token;
use fpas_parser::{Expr, PostfixOperation};
use paths::Step;
use std::collections::HashSet;

struct Visited {
    target: usize,
    owners: HashSet<(usize, usize, Vec<Step>)>,
}

impl Visited {
    fn new(target: usize) -> Self {
        Self {
            target,
            owners: HashSet::new(),
        }
    }
}

/// Finds the source parameter of a static signature, retaining aggregate/payload positions.
pub(super) fn parameter(
    documents: &[NavigationDocument],
    target: usize,
    index: usize,
    symbol: &DocumentSymbol,
    name: &str,
    path: &[Step],
) -> Option<(usize, DocumentSymbol)> {
    parameter_inner(
        documents,
        index,
        symbol,
        name,
        path,
        &mut Visited::new(target),
    )
}

fn parameter_inner(
    documents: &[NavigationDocument],
    index: usize,
    symbol: &DocumentSymbol,
    name: &str,
    path: &[Step],
    visited: &mut Visited,
) -> Option<(usize, DocumentSymbol)> {
    if visited.owners.iter().any(|(owner, offset, previous)| {
        *owner == index && *offset == symbol.selection_span.offset() && previous.len() <= path.len()
    }) || !visited
        .owners
        .insert((index, symbol.selection_span.offset(), path.to_vec()))
    {
        return None;
    }
    if matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
        return sources::declaration_parameter(documents, index, symbol, name, path, visited);
    }
    if matches!(
        symbol.kind,
        SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method
    ) && path.is_empty()
    {
        return symbol
            .children
            .iter()
            .filter(|child| {
                child.kind == SymbolKind::Parameter && child.name.eq_ignore_ascii_case(name)
            })
            .min_by_key(|child| child.selection_span.offset())
            .cloned()
            .map(|child| (index, child));
    }
    let document = &documents[index];
    let tokens = document
        .tokens
        .iter()
        .filter(|token| {
            token.span.offset >= symbol.selection_span.end()
                && token.span.offset < symbol.full_span.end()
        })
        .collect::<Vec<_>>();
    let assignment = tokens
        .iter()
        .position(|token| token.token == Token::ColonAssign);
    // A written receiving annotation is authoritative, including identical implementation labels.
    if let Some(colon) = tokens.iter().position(|token| token.token == Token::Colon)
        && colon < assignment.unwrap_or(tokens.len())
    {
        let start = tokens.get(colon + 1)?.span.offset;
        let end = assignment.map_or(symbol.full_span.end(), |position| {
            tokens[position].span.offset
        });
        let text = document
            .snapshot
            .source()
            .get(start..end)?
            .trim()
            .trim_end_matches(';');
        let (syntax, errors) = fpas_parser::parse_type_expression(text);
        if !errors.is_empty() {
            return None;
        }
        return sources::type_parameter(documents, index, &syntax, start, name, path, visited);
    }
    let Some(assignment) = assignment else {
        let (selector, start, mut payload_path) =
            paths::pattern_selector(document, symbol.full_span.offset())?;
        payload_path.extend_from_slice(path);
        return expression_parameter(
            documents,
            index,
            &selector,
            start,
            name,
            &payload_path,
            visited,
        );
    };
    let start = tokens.get(assignment + 1)?.span.offset;
    let text = document
        .snapshot
        .source()
        .get(start..symbol.full_span.end())?
        .trim()
        .trim_end_matches(';');
    let (value, errors) = fpas_parser::parse_expression(text);
    if !errors.is_empty() {
        return None;
    }
    expression_parameter(documents, index, &value, start, name, path, visited)
}

fn expression_parameter(
    documents: &[NavigationDocument],
    index: usize,
    value: &Expr,
    start: usize,
    name: &str,
    path: &[Step],
    visited: &mut Visited,
) -> Option<(usize, DocumentSymbol)> {
    match value {
        Expr::Designator(designator) | Expr::Call { designator, .. } => {
            let first_index = designator
                .parts
                .iter()
                .position(|part| matches!(part, fpas_parser::DesignatorPart::Index(..)))
                .unwrap_or(designator.parts.len());
            let mut value_path = designator.parts[first_index..]
                .iter()
                .map(|part| match part {
                    fpas_parser::DesignatorPart::Index(..) => Step::Element,
                    fpas_parser::DesignatorPart::Ident(name, _) => Step::Field {
                        variant: None,
                        name: Some(name.clone()),
                        position: 0,
                    },
                })
                .collect::<Vec<_>>();
            value_path.extend_from_slice(path);
            let offset =
                designator.parts[..first_index]
                    .iter()
                    .rev()
                    .find_map(|part| match part {
                        fpas_parser::DesignatorPart::Ident(_, span) => Some(start + span.offset),
                        _ => None,
                    })?;
            let (owner, symbol, _) = resolve(documents, index, offset)?;
            if matches!(value, Expr::Call { .. })
                && matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method)
            {
                return sources::return_parameter(
                    documents,
                    owner,
                    &symbol,
                    name,
                    &value_path,
                    visited,
                );
            }
            if matches!(value, Expr::Call { .. }) && symbol.kind != SymbolKind::Type {
                value_path.insert(designator.parts.len() - first_index, Step::Return);
            }
            parameter_inner(documents, owner, &symbol, name, &value_path, visited)
        }
        Expr::Paren(inner, _) | Expr::NamedArgument { value: inner, .. } => {
            expression_parameter(documents, index, inner, start, name, path, visited)
        }
        Expr::OptionSome(inner, _) | Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) => {
            let expected = match value {
                Expr::OptionSome(..) => Step::Some,
                Expr::ResultOk(..) => Step::Ok,
                _ => Step::Error,
            };
            if path.first() != Some(&expected) {
                return None;
            }
            expression_parameter(documents, index, inner, start, name, &path[1..], visited)
        }
        Expr::ArrayLiteral(values, _) => {
            if path.first() != Some(&Step::Element) {
                return None;
            }
            values.iter().find_map(|value| {
                expression_parameter(documents, index, value, start, name, &path[1..], visited)
            })
        }
        Expr::DictLiteral(values, _) => {
            if path.first() != Some(&Step::Element) {
                return None;
            }
            values.iter().find_map(|(_, value)| {
                expression_parameter(documents, index, value, start, name, &path[1..], visited)
            })
        }
        Expr::If {
            branches,
            else_value,
            ..
        } => branches
            .iter()
            .map(|branch| &branch.value)
            .chain(std::iter::once(else_value.as_ref()))
            .find_map(|value| {
                expression_parameter(documents, index, value, start, name, path, visited)
            }),
        Expr::Case { arms, else_arm, .. } => arms
            .iter()
            .map(|arm| &arm.value)
            .chain(else_arm.iter().map(|arm| &arm.value))
            .find_map(|value| {
                expression_parameter(documents, index, value, start, name, path, visited)
            }),
        Expr::Closure(closure) => {
            if let Some((Step::Return, rest)) = path.split_first() {
                return sources::type_parameter(
                    documents,
                    index,
                    closure.return_type.as_ref()?,
                    start,
                    name,
                    rest,
                    visited,
                );
            }
            if !path.is_empty() {
                return None;
            }
            let param = closure
                .params
                .iter()
                .find(|param| param.name.eq_ignore_ascii_case(name))?;
            sources::source_parameter(documents, index, param, start)
        }
        Expr::Postfix {
            base, operations, ..
        } => {
            let mut value_path = Vec::new();
            for operation in operations {
                match operation {
                    PostfixOperation::Index { .. } => value_path.push(Step::Element),
                    PostfixOperation::Field { name, .. } => value_path.push(Step::Field {
                        variant: None,
                        name: Some(name.clone()),
                        position: 0,
                    }),
                    _ => return None,
                }
            }
            value_path.extend_from_slice(path);
            expression_parameter(documents, index, base, start, name, &value_path, visited)
        }
        _ => None,
    }
}

/// Finds the callable before `(`, stepping over all indexing suffixes.
pub(crate) fn callable_before(document: &NavigationDocument, parenthesis: usize) -> Option<usize> {
    let mut index = parenthesis.checked_sub(1)?;
    while document.tokens.get(index)?.token == Token::RBracket {
        let mut depth = 1usize;
        while depth != 0 {
            index = index.checked_sub(1)?;
            match document.tokens[index].token {
                Token::RBracket => depth += 1,
                Token::LBracket => depth -= 1,
                _ => {}
            }
        }
        index = index.checked_sub(1)?;
    }
    super::token_name(document, index).map(|_| index)
}

/// Finds the start of a qualified or indexed callable receiving expression.
pub(crate) fn expression_start(document: &NavigationDocument, parenthesis: usize) -> Option<usize> {
    let mut index = parenthesis.checked_sub(1)?;
    loop {
        while document.tokens.get(index)?.token == Token::RBracket {
            let mut depth = 1usize;
            while depth != 0 {
                index = index.checked_sub(1)?;
                match document.tokens[index].token {
                    Token::RBracket => depth += 1,
                    Token::LBracket => depth -= 1,
                    _ => {}
                }
            }
            index = index.checked_sub(1)?;
        }
        super::token_name(document, index)?;
        if index > 0 && document.tokens[index - 1].token == Token::Dot {
            index = index.checked_sub(2)?;
        } else {
            return Some(index);
        }
    }
}

/// Resolves a callable field after an indexed receiver to its written label owner.
pub(super) fn expression_label(
    documents: &[NavigationDocument],
    index: usize,
    parenthesis: usize,
    name: &str,
) -> Option<(usize, DocumentSymbol)> {
    let document = &documents[index];
    let start = document.tokens[expression_start(document, parenthesis)?]
        .span
        .offset;
    let (value, errors) = fpas_parser::parse_expression(
        document
            .snapshot
            .source()
            .get(start..document.tokens[parenthesis].span.offset)?
            .trim(),
    );
    if !errors.is_empty() {
        return None;
    }
    expression_parameter(
        documents,
        index,
        &value,
        start,
        name,
        &[],
        &mut Visited::new(index),
    )
}

/// Builds the static element path between a callee name and its call parenthesis.
pub(super) fn call_path(
    document: &NavigationDocument,
    callable: usize,
    parenthesis: usize,
) -> Vec<Step> {
    let mut depth = 0usize;
    let mut path = Vec::new();
    for token in &document.tokens[callable + 1..parenthesis] {
        match token.token {
            Token::LBracket => {
                if depth == 0 {
                    path.push(Step::Element);
                }
                depth += 1;
            }
            Token::RBracket => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    path
}

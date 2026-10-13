//! Resolution of `Name := Value` call-argument labels to callee parameters and
//! enum variant fields.
//!
//! **Documentation:** `docs/pascal/language/functions/parameters.md`

use fpas_lexer::Token;

use super::NavigationDocument;
use super::resolve::resolve;
use crate::{DocumentSymbol, SymbolKind};

/// Outcome of inspecting a token as a named-argument label.
pub(super) enum NamedArgumentLabel {
    /// The token is not a named-argument label.
    NotLabel,
    /// The label names this parameter or variant field of the called target.
    Parameter(usize, Box<DocumentSymbol>),
    /// The label belongs to a call whose parameter cannot be resolved.
    Unresolved,
}

/// Classifies `token_index` as a named-argument label and resolves its parameter.
pub(super) fn resolve_named_argument(
    documents: &[NavigationDocument],
    target_index: usize,
    token_index: usize,
    name: &str,
) -> NamedArgumentLabel {
    let Some(document) = documents.get(target_index) else {
        return NamedArgumentLabel::NotLabel;
    };
    let tokens = &document.tokens;
    if !is_named_argument_label(document, token_index) {
        return NamedArgumentLabel::NotLabel;
    }
    let Some(parenthesis) = enclosing_parenthesis(document, token_index) else {
        return NamedArgumentLabel::Unresolved;
    };
    let Some(callable) = super::callable_labels::callable_before(document, parenthesis) else {
        return NamedArgumentLabel::Unresolved;
    };
    let static_signature = crate::intellisense::callable_values::signature(
        documents,
        target_index,
        callable,
        parenthesis,
        tokens[token_index].span.offset,
    );
    let Some((mut index, mut callee, _)) =
        resolve(documents, target_index, tokens[callable].span.offset)
    else {
        if static_signature.as_ref().is_some_and(|signature| {
            signature.parameters.iter().any(|parameter| {
                crate::documentation::parameter_name(parameter)
                    .is_some_and(|declared| declared.eq_ignore_ascii_case(name))
            })
        }) && let Some((owner, parameter)) =
            super::callable_labels::expression_label(documents, target_index, parenthesis, name)
        {
            return NamedArgumentLabel::Parameter(owner, Box::new(parameter));
        }
        return NamedArgumentLabel::Unresolved;
    };
    if callee.kind == SymbolKind::Type
        && callee
            .children
            .iter()
            .any(|child| child.kind == SymbolKind::Field)
    {
        let Some((owner, record)) =
            super::record_construction::constructor_record(documents, target_index, index, callee)
        else {
            return NamedArgumentLabel::Unresolved;
        };
        index = owner;
        callee = record;
    } else if callee.callable.is_none()
        && let Some((owner, signature)) =
            callable_type_owner(documents, target_index, index, &callee)
    {
        index = owner;
        callee = signature;
    }
    if callee.kind == SymbolKind::Type
        && callee
            .children
            .iter()
            .any(|child| child.kind == SymbolKind::Field)
    {
        let Some((owner, record)) =
            super::record_construction::constructor_record(documents, target_index, index, callee)
        else {
            return NamedArgumentLabel::Unresolved;
        };
        index = owner;
        callee = record;
    }
    let signature = static_signature.as_ref().or(callee.callable.as_ref());
    if signature.is_some_and(|signature| {
        !signature.parameters.iter().any(|parameter| {
            crate::documentation::parameter_name(parameter)
                .is_some_and(|declared| declared.eq_ignore_ascii_case(name))
        })
    }) {
        return NamedArgumentLabel::Unresolved;
    }
    let field_constructor = callee.kind == SymbolKind::EnumMember
        || (callee.kind == SymbolKind::Type
            && callee
                .children
                .iter()
                .any(|child| child.kind == SymbolKind::Field));
    if !field_constructor
        && let Some((owner, parameter)) = super::callable_labels::parameter(
            documents,
            target_index,
            index,
            &callee,
            name,
            &super::callable_labels::call_path(document, callable, parenthesis),
        )
    {
        return NamedArgumentLabel::Parameter(owner, Box::new(parameter));
    }
    callee
        .children
        .into_iter()
        .find(|child| {
            matches!(child.kind, SymbolKind::Parameter | SymbolKind::Field)
                && child.name.eq_ignore_ascii_case(name)
        })
        .map_or(NamedArgumentLabel::Unresolved, |parameter| {
            NamedArgumentLabel::Parameter(index, Box::new(parameter))
        })
}

/// Finds a callable alias in its declaring import environment, preserving its labels.
pub(crate) fn callable_type_owner(
    documents: &[NavigationDocument],
    target: usize,
    index: usize,
    symbol: &DocumentSymbol,
) -> Option<(usize, DocumentSymbol)> {
    let (owner, declaration) =
        super::find_type(documents, target, index, symbol.type_name.as_deref()?)?;
    let (owner, declaration) = super::member_type(documents, owner, declaration)?;
    declaration.callable.as_ref()?;
    Some((owner, declaration.clone()))
}

/// Returns whether `token_index` is the name of a `Name := Value` call argument.
pub(crate) fn is_named_argument_label(document: &NavigationDocument, token_index: usize) -> bool {
    let tokens = &document.tokens;
    matches!(
        tokens.get(token_index + 1).map(|token| &token.token),
        Some(Token::ColonAssign)
    ) && token_index
        .checked_sub(1)
        .is_some_and(|previous| matches!(tokens[previous].token, Token::LParen | Token::Comma))
}

/// Returns the parenthesis enclosing the argument label at `token_index`.
fn enclosing_parenthesis(document: &NavigationDocument, token_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for index in (0..token_index).rev() {
        match document.tokens[index].token {
            Token::RParen | Token::RBracket => depth += 1,
            Token::LBracket => depth = depth.checked_sub(1)?,
            Token::LParen if depth == 0 => return Some(index),
            Token::LParen => depth -= 1,
            _ => {}
        }
    }
    None
}

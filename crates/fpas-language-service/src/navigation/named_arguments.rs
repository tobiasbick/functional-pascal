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
    let Some(callable) = enclosing_callable(document, token_index) else {
        return NamedArgumentLabel::Unresolved;
    };
    let Some((mut index, mut callee, _)) =
        resolve(documents, target_index, tokens[callable].span.offset)
    else {
        return NamedArgumentLabel::Unresolved;
    };
    if callee.kind == SymbolKind::Type {
        let Some((owner, record)) =
            super::record_construction::constructor_record(documents, target_index, index, callee)
        else {
            return NamedArgumentLabel::Unresolved;
        };
        index = owner;
        callee = record;
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

/// Returns the callable token before the parenthesis enclosing `token_index`.
fn enclosing_callable(document: &NavigationDocument, token_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for index in (0..token_index).rev() {
        match document.tokens[index].token {
            Token::RParen | Token::RBracket => depth += 1,
            Token::LBracket => depth = depth.checked_sub(1)?,
            Token::LParen if depth == 0 => return index.checked_sub(1),
            Token::LParen => depth -= 1,
            _ => {}
        }
    }
    None
}

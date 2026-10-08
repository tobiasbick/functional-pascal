//! Field-label completion using the same constructor resolution as navigation.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::{
    CompletionCandidate, CompletionSource, context::CompletionContext, signature_help::active_call,
};
use crate::{
    SymbolKind,
    navigation::{NavigationDocument, record_construction::constructor_record, resolve},
};
use fpas_lexer::Token;
use std::collections::HashSet;

/// Suggest unfilled named fields at an argument start, preserving existing `:=` tokens.
pub(super) fn field_completions(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
    context: &CompletionContext,
) -> Option<Vec<CompletionCandidate>> {
    if context.receiver.is_some() {
        return None;
    }
    let document = &documents[target_index];
    let frame = active_call(document, offset)?;
    if document
        .tokens
        .iter()
        .skip(frame.argument_start)
        .take_while(|token| token.span.offset < context.replacement.offset())
        .next()
        .is_some()
    {
        return None;
    }
    let (index, callee, _) = resolve(
        documents,
        target_index,
        document.tokens[frame.callable_token].span.offset,
    )?;
    let (index, record) = constructor_record(documents, target_index, index, callee)?;
    let mut provided = HashSet::new();
    let mut depth = 0usize;
    let mut has_assignment = false;
    for (index, token) in document
        .tokens
        .iter()
        .enumerate()
        .skip(frame.parenthesis + 1)
    {
        match token.token {
            Token::RParen if depth == 0 => break,
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            Token::ColonAssign if depth == 0 => {
                let name = document.tokens.get(index.checked_sub(1)?)?;
                if let Token::Ident(name_text) = &name.token {
                    if name.span.offset == context.replacement.offset() {
                        has_assignment = true;
                    } else {
                        provided.insert(name_text.to_ascii_lowercase());
                    }
                }
            }
            _ => {}
        }
    }
    let mut candidates = record
        .children
        .iter()
        .filter(|field| {
            field.kind == SymbolKind::Field
                && !provided.contains(&field.name.to_ascii_lowercase())
                && field
                    .name
                    .to_ascii_lowercase()
                    .starts_with(&context.prefix.to_ascii_lowercase())
        })
        .map(|field| {
            let mut candidate = super::completion::declaration_candidate(
                documents,
                index,
                field,
                context.replacement,
                CompletionSource::Declaration,
                0,
                None,
            );
            if !has_assignment {
                candidate.insert_text.push_str(" := ");
            }
            candidate
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.sort_text.cmp(&right.sort_text));
    Some(candidates)
}

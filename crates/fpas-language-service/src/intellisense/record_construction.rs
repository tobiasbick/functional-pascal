//! Complete static argument and pattern labels using their declaration owners.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::{
    CompletionCandidate, CompletionKind, CompletionSource,
    context::CompletionContext,
    signature_help::{active_call, payload_signature},
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
    if has_positional_arguments(document, frame.parenthesis, context.replacement.offset()) {
        return None;
    }
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
    let mut resolved = resolve(
        documents,
        target_index,
        document.tokens[frame.callable_token].span.offset,
    )
    .map(|(index, callee, _)| (index, callee));
    if let Some((index, callee)) = &resolved {
        if callee.kind == SymbolKind::Type
            && callee
                .children
                .iter()
                .any(|child| child.kind == SymbolKind::Field)
        {
            resolved = constructor_record(documents, target_index, *index, callee.clone());
        } else if callee.callable.is_none()
            && let Some(owner) = crate::navigation::named_arguments::callable_type_owner(
                documents,
                target_index,
                *index,
                callee,
            )
        {
            resolved = Some(owner);
        }
    }
    if let Some((index, callee)) = &resolved
        && callee.kind == SymbolKind::Type
        && callee
            .children
            .iter()
            .any(|child| child.kind == SymbolKind::Field)
    {
        resolved = Some(constructor_record(
            documents,
            target_index,
            *index,
            callee.clone(),
        )?);
    }
    let signature = super::callable_values::signature(
        documents,
        target_index,
        frame.callable_token,
        frame.parenthesis,
        offset,
    )
    .or_else(|| {
        resolved
            .as_ref()
            .and_then(|(_, symbol)| symbol.callable.clone())
    })
    .or_else(|| payload_signature(&document.tokens[frame.callable_token].token))?;
    if signature.parameters.is_empty() {
        return None;
    }
    let Some((index, callee)) = resolved.filter(|(_, callee)| {
        callee
            .children
            .iter()
            .any(|field| matches!(field.kind, SymbolKind::Field | SymbolKind::Parameter))
    }) else {
        return Some(
            signature
                .parameters
                .iter()
                .filter_map(|parameter| {
                    let name = crate::documentation::parameter_name(parameter)?;
                    if provided.contains(&name.to_ascii_lowercase())
                        || !name
                            .to_ascii_lowercase()
                            .starts_with(&context.prefix.to_ascii_lowercase())
                    {
                        return None;
                    }
                    Some(CompletionCandidate {
                        label: name.into(),
                        kind: CompletionKind::Symbol(SymbolKind::Parameter),
                        detail: parameter.clone(),
                        owner: None,
                        qualified_name: name.into(),
                        sort_text: name.into(),
                        filter_text: name.into(),
                        insert_text: format!("{name}{}", if has_assignment { "" } else { " := " }),
                        replacement_span: context.replacement,
                        source: CompletionSource::Declaration,
                        inline_documentation: None,
                        documentation: None,
                        additional_edit: None,
                    })
                })
                .collect(),
        );
    };
    let mut candidates = callee
        .children
        .iter()
        .filter(|field| {
            matches!(field.kind, SymbolKind::Field | SymbolKind::Parameter)
                && (!matches!(
                    callee.kind,
                    SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method
                ) || field.scope_span == callee.full_span)
                && signature.parameters.iter().any(|parameter| {
                    crate::documentation::parameter_name(parameter)
                        .is_some_and(|name| name.eq_ignore_ascii_case(&field.name))
                })
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

fn has_positional_arguments(
    document: &NavigationDocument,
    parenthesis: usize,
    current: usize,
) -> bool {
    let mut depth = 0usize;
    let mut start = None;
    let mut named = false;
    for token in document.tokens.iter().skip(parenthesis + 1) {
        match token.token {
            Token::Comma | Token::RParen if depth == 0 => {
                if start.is_some_and(|start| !(start <= current && current <= token.span.offset))
                    && !named
                {
                    return true;
                }
                if token.token == Token::RParen {
                    break;
                }
                start = None;
                named = false;
            }
            Token::ColonAssign if depth == 0 => named = true,
            Token::LParen | Token::LBracket => {
                start.get_or_insert(token.span.offset);
                depth += 1;
            }
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            _ => {
                start.get_or_insert(token.span.offset);
            }
        }
    }
    false
}

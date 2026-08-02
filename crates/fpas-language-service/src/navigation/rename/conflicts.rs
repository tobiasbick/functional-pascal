//! Lexical binding preservation for symbol rename.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use super::RenameError;
use crate::navigation::NavigationDocument;
use crate::navigation::references::{ReferenceLocation, ResolvedTarget, resolve_target};
use crate::navigation::resolve::resolve_unqualified;
use crate::{CancellationToken, DocumentSymbol};

pub(super) fn reject_resolution_conflicts(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
    references: &[ReferenceLocation],
    cancellation: &CancellationToken,
) -> Result<(), RenameError> {
    reject_same_scope_conflict(documents, target, new_name)?;

    let mut renamed = documents.to_vec();
    rename_target_symbol(&mut renamed, target, new_name);
    ensure_edited_references_still_bind(
        documents,
        &renamed,
        target,
        new_name,
        references,
        cancellation,
    )?;
    ensure_existing_references_still_bind(documents, &renamed, new_name, cancellation)
}

fn reject_same_scope_conflict(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
) -> Result<(), RenameError> {
    let target_document = &documents[target.document_index];
    let conflict = target_document.all_symbols().into_iter().any(|symbol| {
        symbol.selection_span != target.symbol.selection_span
            && symbol.name.eq_ignore_ascii_case(new_name)
            && symbol.scope_span == target.symbol.scope_span
    });
    if conflict {
        conflict_error(new_name)
    } else {
        Ok(())
    }
}

fn ensure_edited_references_still_bind(
    documents: &[NavigationDocument],
    renamed: &[NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
    references: &[ReferenceLocation],
    cancellation: &CancellationToken,
) -> Result<(), RenameError> {
    for reference in references {
        cancellation.check()?;
        let Some(document_index) = documents
            .iter()
            .position(|document| document.path == reference.path)
        else {
            return conflict_error(new_name);
        };
        if !is_unqualified(&documents[document_index], reference.span) {
            continue;
        }
        let resolved =
            resolve_unqualified(renamed, document_index, new_name, reference.span.offset);
        if !resolved.is_some_and(|resolved| same_declaration(resolved, target)) {
            return conflict_error(new_name);
        }
    }
    Ok(())
}

fn ensure_existing_references_still_bind(
    documents: &[NavigationDocument],
    renamed: &[NavigationDocument],
    new_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), RenameError> {
    for (document_index, document) in documents.iter().enumerate() {
        cancellation.check()?;
        for token in &document.tokens {
            cancellation.check()?;
            let Token::Ident(name) = &token.token else {
                continue;
            };
            let span = SourceSpan::from(token.span);
            if !name.eq_ignore_ascii_case(new_name) || !is_unqualified(document, span) {
                continue;
            }
            let Some(before) = resolve_target(documents, document_index, token.span.offset) else {
                continue;
            };
            let after = resolve_unqualified(renamed, document_index, new_name, token.span.offset);
            if !after.is_some_and(|resolved| {
                resolved.0 == before.document_index
                    && resolved.1.selection_span == before.symbol.selection_span
            }) {
                return conflict_error(new_name);
            }
        }
    }
    Ok(())
}

fn rename_target_symbol(
    documents: &mut [NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
) {
    for symbol in &mut documents[target.document_index].roots {
        if let Some(symbol) = find_symbol_mut(symbol, target.symbol.selection_span) {
            symbol.name = new_name.to_owned();
            if let Some((owner, _)) = symbol.qualified_name.rsplit_once('.') {
                symbol.qualified_name = format!("{owner}.{new_name}");
            } else {
                symbol.qualified_name = new_name.to_owned();
            }
            return;
        }
    }
}

fn find_symbol_mut(
    symbol: &mut DocumentSymbol,
    selection_span: SourceSpan,
) -> Option<&mut DocumentSymbol> {
    if symbol.selection_span == selection_span {
        return Some(symbol);
    }
    symbol
        .children
        .iter_mut()
        .find_map(|child| find_symbol_mut(child, selection_span))
}

fn is_unqualified(document: &NavigationDocument, span: SourceSpan) -> bool {
    let Some(index) = document
        .tokens
        .iter()
        .position(|token| token.span.offset == span.offset && token.span.length == span.length)
    else {
        return false;
    };
    index == 0 || !matches!(document.tokens[index - 1].token, Token::Dot)
}

fn same_declaration(resolved: (usize, DocumentSymbol), target: &ResolvedTarget) -> bool {
    resolved.0 == target.document_index && resolved.1.selection_span == target.symbol.selection_span
}

fn conflict_error(name: &str) -> Result<(), RenameError> {
    Err(RenameError::Conflict {
        name: name.to_owned(),
    })
}

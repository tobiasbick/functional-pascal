//! Lexical, import, and member resolution for editor queries.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use super::NavigationDocument;
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

pub(crate) fn resolve(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
) -> Option<(usize, DocumentSymbol, SourceSpan)> {
    let target = documents.get(target_index)?;
    let (token_index, name, range) = identifier_at(target, offset)?;

    if let Some(symbol) = target
        .all_symbols()
        .into_iter()
        .find(|symbol| contains(symbol.selection_span, offset))
    {
        return Some((target_index, symbol.clone(), range));
    }

    let (parts, selected_part) = qualified_parts(target, token_index);
    if selected_part == 0 {
        return resolve_unqualified(documents, target_index, &name, offset)
            .map(|(index, symbol)| (index, symbol, range));
    }

    resolve_qualified(documents, target_index, &parts[..=selected_part], offset)
        .map(|(index, symbol)| (index, symbol, range))
}

fn identifier_at(
    document: &NavigationDocument,
    offset: usize,
) -> Option<(usize, String, SourceSpan)> {
    document
        .tokens
        .iter()
        .enumerate()
        .find_map(|(index, token)| {
            let name = token_name(document, index)?;
            let end = token.span.offset.saturating_add(token.span.length);
            (token.span.offset <= offset && offset < end)
                .then(|| (index, name, token.span.diagnostic_span_or_synthetic()))
        })
}

fn qualified_parts(document: &NavigationDocument, selected: usize) -> (Vec<String>, usize) {
    let mut start = selected;
    while start >= 2
        && matches!(document.tokens[start - 1].token, Token::Dot)
        && token_name(document, start - 2).is_some()
    {
        start -= 2;
    }
    let mut parts = Vec::new();
    let mut selected_part = 0;
    let mut index = start;
    while let Some(name) = token_name(document, index) {
        if index == selected {
            selected_part = parts.len();
        }
        parts.push(name);
        if index + 2 >= document.tokens.len()
            || !matches!(document.tokens[index + 1].token, Token::Dot)
            || token_name(document, index + 2).is_none()
        {
            break;
        }
        index += 2;
    }
    (parts, selected_part)
}

pub(crate) fn token_name(document: &NavigationDocument, index: usize) -> Option<String> {
    let token = document.tokens.get(index)?;
    if let Token::Ident(name) = &token.token {
        return Some(name.clone());
    }
    let end = token.span.offset.checked_add(token.span.length)?;
    let value = document.snapshot.source().get(token.span.offset..end)?;
    let mut chars = value.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic() || first == '_')
        .then_some(())
        .filter(|()| chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
        .map(|()| value.to_owned())
}

pub(crate) fn resolve_unqualified(
    documents: &[NavigationDocument],
    target_index: usize,
    name: &str,
    offset: usize,
) -> Option<(usize, DocumentSymbol)> {
    let target = documents.get(target_index)?;
    let mut local = target
        .all_symbols()
        .into_iter()
        .filter(|symbol| symbol.name.eq_ignore_ascii_case(name))
        .filter(|symbol| unqualified_kind(symbol.kind))
        .filter(|symbol| contains(symbol.scope_span, offset))
        .filter(|symbol| symbol.visible_from <= offset)
        .collect::<Vec<_>>();
    local.sort_by(|left, right| {
        left.scope_span
            .length()
            .cmp(&right.scope_span.length())
            .then_with(|| right.visible_from.cmp(&left.visible_from))
    });
    if let Some(symbol) = local.first() {
        return Some((target_index, (*symbol).clone()));
    }

    let imported = imported_top_level(documents, target_index)
        .into_iter()
        .filter(|(_, symbol)| symbol.name.eq_ignore_ascii_case(name))
        .collect::<Vec<_>>();
    (imported.len() == 1).then(|| {
        let (index, symbol) = imported[0];
        (index, symbol.clone())
    })
}

pub(crate) fn resolve_qualified(
    documents: &[NavigationDocument],
    target_index: usize,
    parts: &[String],
    offset: usize,
) -> Option<(usize, DocumentSymbol)> {
    let target = documents.get(target_index)?;
    let first = parts.first()?;
    let owner_candidates = documents
        .iter()
        .enumerate()
        .filter_map(|(index, document)| {
            let owner_parts = document.owner.split('.').count();
            (parts.len() >= owner_parts
                && parts[..owner_parts]
                    .join(".")
                    .eq_ignore_ascii_case(&document.owner)
                && target.uses_owner(&document.owner))
            .then_some((index, document, owner_parts))
        })
        .collect::<Vec<_>>();
    if !owner_candidates.is_empty() {
        let qualified = parts.join(".");
        let mut resolved = owner_candidates
            .into_iter()
            .filter_map(|(index, document, owner_parts)| {
                let symbol = if parts.len() == owner_parts {
                    document.roots.first()
                } else {
                    document.all_symbols().into_iter().find(|symbol| {
                        symbol.qualified_name.eq_ignore_ascii_case(&qualified)
                            && (index == target_index
                                || symbol.visibility == SymbolVisibility::Public)
                    })
                }?;
                Some((index, symbol.clone()))
            })
            .collect::<Vec<_>>();
        return (resolved.len() == 1).then(|| resolved.remove(0));
    }

    let (base_index, base) = resolve_unqualified(documents, target_index, first, offset)?;
    let mut owner_type = if matches!(base.kind, SymbolKind::Type | SymbolKind::Enum) {
        Some(base.qualified_name.clone())
    } else {
        base.type_name.clone()
    }?;
    for (member_index, member_name) in parts[1..].iter().enumerate() {
        let (type_index, type_symbol) =
            find_type(documents, target_index, base_index, &owner_type)?;
        let member = type_symbol.children.iter().find(|member| {
            member.name.eq_ignore_ascii_case(member_name)
                && (type_index == target_index || member.visibility == SymbolVisibility::Public)
        })?;
        owner_type = member
            .type_name
            .clone()
            .unwrap_or_else(|| member.qualified_name.clone());
        if member_index + 2 == parts.len() {
            return Some((type_index, member.clone()));
        }
    }
    None
}

pub(crate) fn find_type<'a>(
    documents: &'a [NavigationDocument],
    target_index: usize,
    preferred_index: usize,
    name: &str,
) -> Option<(usize, &'a DocumentSymbol)> {
    let short = name.rsplit('.').next().unwrap_or(name);
    let preferred = &documents[preferred_index];
    if let Some(symbol) = preferred.top_level().iter().find(|symbol| {
        matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum)
            && symbol.name.eq_ignore_ascii_case(short)
    }) {
        return Some((preferred_index, symbol));
    }
    documents.iter().enumerate().find_map(|(index, document)| {
        if index != target_index && !documents[target_index].uses_owner(&document.owner) {
            return None;
        }
        document
            .top_level()
            .iter()
            .find(|symbol| {
                matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum)
                    && (symbol.name.eq_ignore_ascii_case(short)
                        || symbol.qualified_name.eq_ignore_ascii_case(name))
                    && (index == target_index || symbol.visibility == SymbolVisibility::Public)
            })
            .map(|symbol| (index, symbol))
    })
}

fn imported_top_level(
    documents: &[NavigationDocument],
    target_index: usize,
) -> Vec<(usize, &DocumentSymbol)> {
    let target = &documents[target_index];
    documents
        .iter()
        .enumerate()
        .filter(|(index, document)| *index != target_index && target.uses_owner(&document.owner))
        .flat_map(|(index, document)| {
            document
                .top_level()
                .iter()
                .filter(|symbol| symbol.visibility == SymbolVisibility::Public)
                .map(move |symbol| (index, symbol))
        })
        .collect()
}

fn unqualified_kind(kind: SymbolKind) -> bool {
    !matches!(
        kind,
        SymbolKind::Program
            | SymbolKind::Unit
            | SymbolKind::Field
            | SymbolKind::Property
            | SymbolKind::Event
    )
}

fn contains(span: SourceSpan, offset: usize) -> bool {
    span.offset() <= offset && offset < span.end()
}

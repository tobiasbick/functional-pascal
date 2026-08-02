//! Rich completion candidates over lexical scopes, imports, members, and keywords.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::auto_import::auto_import_candidates;
use super::context::completion_context;
use super::{CompletionCandidate, CompletionDocumentation, CompletionKind, CompletionSource};
use crate::navigation::{
    NavigationDocument, NavigationResult, find_type, resolve_qualified, resolve_unqualified,
};
use crate::{DocumentSymbol, LanguageService, LanguageServiceError, SymbolKind, SymbolVisibility};

impl LanguageService {
    /// Returns ranked declarations, keywords, and safe imports for a UTF-8 byte offset.
    pub fn completions(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Vec<CompletionCandidate>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context
            .target_index
            .and_then(|target_index| {
                let completion = completion_context(&context.documents[target_index], offset)?;
                Some(complete(
                    &context.documents,
                    target_index,
                    offset,
                    completion,
                ))
            })
            .unwrap_or_default();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }
}

fn complete(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
    context: super::context::CompletionContext,
) -> Vec<CompletionCandidate> {
    let symbols = if let Some(receiver) = &context.receiver {
        member_candidates(documents, target_index, receiver, offset)
    } else {
        visible_candidates(documents, target_index, offset)
    };
    let mut candidates = symbols
        .into_iter()
        .filter(|(_, symbol)| starts_with(&symbol.name, &context.prefix))
        .map(|(document_index, symbol)| {
            declaration_candidate(
                documents,
                document_index,
                symbol,
                context.replacement,
                CompletionSource::Declaration,
                0,
                None,
            )
        })
        .collect::<Vec<_>>();

    if context.receiver.is_none() {
        candidates.extend(keyword_candidates(&context));
        if !context.prefix.is_empty() {
            let visible_labels = candidates
                .iter()
                .filter(|candidate| candidate.source == CompletionSource::Declaration)
                .map(|candidate| candidate.label.to_ascii_lowercase())
                .collect::<HashSet<_>>();
            candidates.extend(
                auto_import_candidates(documents, target_index, &context.prefix, &visible_labels)
                    .into_iter()
                    .map(|candidate| {
                        declaration_candidate(
                            documents,
                            candidate.document_index,
                            candidate.symbol,
                            context.replacement,
                            CompletionSource::AutoImport,
                            2,
                            Some(candidate.edit),
                        )
                    }),
            );
        }
    }

    candidates.sort_by(|left, right| left.sort_text.cmp(&right.sort_text));
    candidates.dedup_by(|left, right| {
        left.label.eq_ignore_ascii_case(&right.label)
            && left
                .qualified_name
                .eq_ignore_ascii_case(&right.qualified_name)
            && left.source == right.source
    });
    candidates
}

fn visible_candidates(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
) -> Vec<(usize, &DocumentSymbol)> {
    let target = &documents[target_index];
    let mut nearest = HashMap::<String, &DocumentSymbol>::new();
    let mut local = target
        .all_symbols()
        .into_iter()
        .filter(|symbol| unqualified_kind(symbol.kind))
        .filter(|symbol| contains(symbol.scope_span, offset))
        .filter(|symbol| symbol.visible_from <= offset)
        .collect::<Vec<_>>();
    local.sort_by(|left, right| {
        left.scope_span
            .length
            .cmp(&right.scope_span.length)
            .then_with(|| right.visible_from.cmp(&left.visible_from))
    });
    for symbol in local {
        nearest
            .entry(symbol.name.to_ascii_lowercase())
            .or_insert(symbol);
    }
    let mut result = nearest
        .values()
        .map(|symbol| (target_index, *symbol))
        .collect::<Vec<_>>();
    for (document_index, document) in documents.iter().enumerate() {
        if document_index == target_index || !target.uses_owner(&document.owner) {
            continue;
        }
        for symbol in document
            .top_level()
            .iter()
            .filter(|symbol| symbol.visibility == SymbolVisibility::Public)
        {
            if !nearest.contains_key(&symbol.name.to_ascii_lowercase()) {
                result.push((document_index, symbol));
            }
        }
    }
    result
}

fn member_candidates<'a>(
    documents: &'a [NavigationDocument],
    target_index: usize,
    receiver: &str,
    offset: usize,
) -> Vec<(usize, &'a DocumentSymbol)> {
    if let Some((index, document)) = documents.iter().enumerate().find(|(_, document)| {
        document.owner.eq_ignore_ascii_case(receiver)
            && documents[target_index].uses_owner(&document.owner)
    }) {
        return public_members(document.top_level(), index, target_index);
    }

    let parts = receiver.split('.').map(str::to_owned).collect::<Vec<_>>();
    let resolved = if parts.len() == 1 {
        resolve_unqualified(documents, target_index, &parts[0], offset)
    } else {
        resolve_qualified(documents, target_index, &parts, offset)
    };
    let Some((base_index, base)) = resolved else {
        return Vec::new();
    };
    let Some(type_name) = (if matches!(base.kind, SymbolKind::Type | SymbolKind::Enum) {
        Some(base.qualified_name)
    } else {
        base.type_name
    }) else {
        return Vec::new();
    };
    let Some((type_index, type_symbol)) =
        find_type(documents, target_index, base_index, &type_name)
    else {
        return Vec::new();
    };
    public_members(&type_symbol.children, type_index, target_index)
}

fn public_members(
    members: &[DocumentSymbol],
    document_index: usize,
    target_index: usize,
) -> Vec<(usize, &DocumentSymbol)> {
    members
        .iter()
        .filter(|member| {
            document_index == target_index || member.visibility == SymbolVisibility::Public
        })
        .map(|member| (document_index, member))
        .collect()
}

fn declaration_candidate(
    documents: &[NavigationDocument],
    document_index: usize,
    symbol: &DocumentSymbol,
    replacement_span: fpas_diagnostics::SourceSpan,
    source: CompletionSource,
    rank: u8,
    additional_edit: Option<super::CompletionEdit>,
) -> CompletionCandidate {
    let owner = symbol
        .qualified_name
        .strip_suffix(&format!(".{}", symbol.name))
        .map(str::to_owned);
    CompletionCandidate {
        label: symbol.name.clone(),
        kind: CompletionKind::Symbol(symbol.kind),
        detail: symbol.detail.clone(),
        owner,
        qualified_name: symbol.qualified_name.clone(),
        sort_text: sort_text(rank, &symbol.name, &symbol.qualified_name),
        filter_text: symbol.name.clone(),
        insert_text: symbol.name.clone(),
        replacement_span,
        source,
        documentation: Some(CompletionDocumentation {
            path: documents[document_index].path.clone(),
            declaration_offset: symbol.full_span.offset,
            source_revision: documents[document_index].snapshot.revision(),
            qualified_name: symbol.qualified_name.clone(),
        }),
        additional_edit,
    }
}

fn keyword_candidates(context: &super::context::CompletionContext) -> Vec<CompletionCandidate> {
    let keywords: &[&str] = if context.statements {
        &[
            "begin", "case", "false", "for", "go", "if", "mutable", "nil", "panic", "repeat",
            "true", "var", "while",
        ]
    } else {
        &[
            "const",
            "function",
            "mutable",
            "procedure",
            "public",
            "type",
            "var",
        ]
    };
    keywords
        .iter()
        .filter(|keyword| starts_with(keyword, &context.prefix))
        .map(|keyword| CompletionCandidate {
            label: (*keyword).to_owned(),
            kind: CompletionKind::Keyword,
            detail: "Functional Pascal keyword".to_owned(),
            owner: None,
            qualified_name: (*keyword).to_owned(),
            sort_text: sort_text(1, keyword, keyword),
            filter_text: (*keyword).to_owned(),
            insert_text: (*keyword).to_owned(),
            replacement_span: context.replacement,
            source: CompletionSource::Keyword,
            documentation: None,
            additional_edit: None,
        })
        .collect()
}

fn sort_text(rank: u8, label: &str, qualified_name: &str) -> String {
    format!(
        "{rank:02}:{}:{}",
        label.to_ascii_lowercase(),
        qualified_name.to_ascii_lowercase()
    )
}

fn starts_with(name: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || name
            .get(..prefix.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
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

fn contains(span: fpas_diagnostics::SourceSpan, offset: usize) -> bool {
    span.offset <= offset && offset < span.offset.saturating_add(span.length)
}

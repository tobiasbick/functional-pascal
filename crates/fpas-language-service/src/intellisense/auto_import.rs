//! Deterministic completion edits for one unambiguous public unit declaration.

use std::collections::{HashMap, HashSet};

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use super::CompletionEdit;
use crate::navigation::NavigationDocument;
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

pub(super) struct AutoImportCandidate<'a> {
    pub(super) document_index: usize,
    pub(super) symbol: &'a DocumentSymbol,
    pub(super) edit: CompletionEdit,
    pub(super) alias: String,
}

pub(super) fn auto_import_candidates<'a>(
    documents: &'a [NavigationDocument],
    target_index: usize,
    prefix: &str,
    visible_labels: &HashSet<String>,
) -> Vec<AutoImportCandidate<'a>> {
    let target = &documents[target_index];
    let mut grouped = HashMap::<String, Vec<(usize, &DocumentSymbol)>>::new();
    for (document_index, document) in documents.iter().enumerate() {
        if document_index == target_index || target.uses_owner(&document.owner) {
            continue;
        }
        for symbol in document.top_level().iter().filter(|symbol| {
            symbol.visibility == SymbolVisibility::Public
                && auto_import_kind(symbol.kind)
                && starts_with(&symbol.name, prefix)
                && !visible_labels.contains(&symbol.name.to_ascii_lowercase())
        }) {
            grouped
                .entry(symbol.name.to_ascii_lowercase())
                .or_default()
                .push((document_index, symbol));
        }
    }

    let mut candidates = grouped
        .into_values()
        .filter_map(|matches| {
            let [(document_index, symbol)] = matches.as_slice() else {
                return None;
            };
            import_edit(target, &documents[*document_index].owner).map(|(edit, alias)| {
                AutoImportCandidate {
                    document_index: *document_index,
                    symbol,
                    edit,
                    alias,
                }
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.symbol
            .name
            .to_ascii_lowercase()
            .cmp(&right.symbol.name.to_ascii_lowercase())
            .then_with(|| left.symbol.qualified_name.cmp(&right.symbol.qualified_name))
    });
    candidates
}

fn import_edit(document: &NavigationDocument, unit: &str) -> Option<(CompletionEdit, String)> {
    let short = unit.rsplit('.').next()?;
    let mut alias = short.to_owned();
    if !matches!(fpas_lexer::lex(&alias).0.first()?.token, Token::Ident(_)) {
        alias.push_str("Unit");
    }
    let base = alias.clone();
    let mut suffix = 2;
    while document.tokens.iter().any(
        |token| matches!(&token.token, Token::Ident(name) if name.eq_ignore_ascii_case(&alias)),
    ) {
        alias = format!("{base}{suffix}");
        suffix += 1;
    }
    let header = document
        .tokens
        .iter()
        .find(|token| token.token == Token::Semicolon)?;
    let mut insertion = document.uses.last().map_or_else(
        || header.span.offset.checked_add(header.span.length),
        |import| import.span.offset.checked_add(import.span.length),
    )?;
    let source = document.snapshot.source();
    let line_end = source[insertion..]
        .find('\n')
        .map_or(source.len(), |length| insertion + length);
    if source[insertion..line_end].trim_start().starts_with("//") {
        insertion = (line_end + 1).min(source.len());
    }
    let edit = CompletionEdit {
        span: SourceSpan::new(insertion, 0, 1, 1),
        new_text: format!(
            "{}uses {unit} as {alias};",
            if document.uses.is_empty() {
                "\n\n"
            } else {
                "\n"
            }
        ),
    };
    Some((edit, alias))
}

fn starts_with(name: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || name
            .get(..prefix.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn auto_import_kind(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::Constant
            | SymbolKind::Variable
            | SymbolKind::MutableVariable
            | SymbolKind::Type
            | SymbolKind::Enum
            | SymbolKind::Function
            | SymbolKind::Procedure
    )
}

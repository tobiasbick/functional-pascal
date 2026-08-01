//! Deterministic completion edits for one unambiguous public unit declaration.

use std::collections::{HashMap, HashSet};

use fpas_diagnostics::SourceSpan;
use fpas_fmt::format_compilation_unit;
use fpas_lexer::{Span, Token};
use fpas_parser::{CompilationUnit, QualifiedId};

use super::CompletionEdit;
use crate::navigation::NavigationDocument;
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

pub(super) struct AutoImportCandidate<'a> {
    pub(super) document_index: usize,
    pub(super) symbol: &'a DocumentSymbol,
    pub(super) edit: CompletionEdit,
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
            import_edit(target, &documents[*document_index].owner).map(|edit| AutoImportCandidate {
                document_index: *document_index,
                symbol,
                edit,
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

fn import_edit(document: &NavigationDocument, unit: &str) -> Option<CompletionEdit> {
    let clause = canonical_uses_clause(document, unit)?;
    let uses_index = document
        .tokens
        .iter()
        .position(|token| matches!(token.token, Token::Uses));
    if let Some(uses_index) = uses_index {
        let semicolon = document.tokens[uses_index..]
            .iter()
            .find(|token| matches!(token.token, Token::Semicolon))?;
        let start = document.tokens[uses_index].span.offset;
        let end = semicolon.span.offset.saturating_add(semicolon.span.length);
        let current = document.snapshot.source().get(start..end)?;
        if contains_comment(current) {
            return None;
        }
        return Some(CompletionEdit {
            span: SourceSpan::new(start, end.saturating_sub(start), 1, 1),
            new_text: clause,
        });
    }

    let header_semicolon = document
        .tokens
        .iter()
        .find(|token| matches!(token.token, Token::Semicolon))?;
    let insertion = header_semicolon
        .span
        .offset
        .saturating_add(header_semicolon.span.length);
    let line_end = document.snapshot.source()[insertion..]
        .find('\n')
        .map_or(document.snapshot.source().len(), |length| {
            insertion + length
        });
    if !document.snapshot.source()[insertion..line_end]
        .trim()
        .is_empty()
    {
        return None;
    }
    Some(CompletionEdit {
        span: SourceSpan::new(insertion, 0, 1, 1),
        new_text: format!("\n\n{clause}"),
    })
}

fn canonical_uses_clause(document: &NavigationDocument, unit: &str) -> Option<String> {
    let mut compilation = document.snapshot.compilation_unit().clone();
    let uses = match &mut compilation {
        CompilationUnit::Program(program) => &mut program.uses,
        CompilationUnit::Unit(unit) => &mut unit.uses,
    };
    uses.push(QualifiedId {
        parts: unit.split('.').map(str::to_owned).collect(),
        span: Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
            source_id: 0,
        },
    });
    let formatted = format_compilation_unit(&compilation);
    let start = formatted.find("uses")?;
    let end = formatted.get(start..)?.find(';')? + start + 1;
    formatted.get(start..end).map(str::to_owned)
}

fn contains_comment(source: &str) -> bool {
    source.contains("//") || source.contains('{') || source.contains("(*")
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

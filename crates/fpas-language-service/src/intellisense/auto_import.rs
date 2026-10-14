//! Deterministic qualified completion edits for public unit declarations.

use std::collections::{HashMap, HashSet};

use fpas_diagnostics::SourceSpan;
use fpas_fmt::format_compilation_unit;
use fpas_lexer::{Span, Token};
use fpas_parser::{CompilationUnit, Import, ImportAlias, QualifiedId};

use super::CompletionEdit;
use super::import_aliases::new_import_alias;
use crate::navigation::NavigationDocument;
use crate::{DocumentSymbol, SymbolKind, SymbolVisibility};

/// A public symbol qualified by a source alias, with an import edit when needed.
pub(super) struct AutoImportCandidate<'a> {
    pub(super) document_index: usize,
    pub(super) symbol: &'a DocumentSymbol,
    pub(super) edit: Option<CompletionEdit>,
    pub(super) namespace: String,
}

/// Reuses direct aliases before offering qualified declarations from new imports.
/// See `docs/pascal/tools/editor-integration.md`.
pub(super) fn auto_import_candidates<'a>(
    documents: &'a [NavigationDocument],
    target_index: usize,
    prefix: &str,
    visible_labels: &HashSet<String>,
) -> Vec<AutoImportCandidate<'a>> {
    let target = &documents[target_index];
    let mut grouped = HashMap::<String, Vec<(usize, &DocumentSymbol)>>::new();
    for (document_index, document) in documents.iter().enumerate() {
        if document_index == target_index {
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
        .flat_map(|matches| {
            let existing = matches
                .iter()
                .filter_map(|(document_index, symbol)| {
                    let alias = target
                        .import_for(&documents[*document_index].owner)?
                        .alias
                        .as_ref()?;
                    Some(AutoImportCandidate {
                        document_index: *document_index,
                        symbol,
                        edit: None,
                        namespace: alias.name.clone(),
                    })
                })
                .collect::<Vec<_>>();
            if !existing.is_empty() {
                return existing;
            }
            matches
                .into_iter()
                .filter_map(|(document_index, symbol)| {
                    let owner = &documents[document_index].owner;
                    let alias = new_import_alias(target, owner);
                    let edit = import_edit(target, owner, &alias)?;
                    Some(AutoImportCandidate {
                        document_index,
                        symbol,
                        edit: Some(edit),
                        namespace: alias,
                    })
                })
                .collect()
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

fn import_edit(document: &NavigationDocument, unit: &str, alias: &str) -> Option<CompletionEdit> {
    let root = unit.split('.').next()?;
    if document
        .uses
        .iter()
        .filter_map(|import| import.alias.as_ref())
        .any(|existing| existing.name.eq_ignore_ascii_case(root))
    {
        return None;
    }
    let clause = canonical_uses_clause(document, unit, alias)?;
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

fn canonical_uses_clause(document: &NavigationDocument, unit: &str, alias: &str) -> Option<String> {
    let mut compilation = document.snapshot.compilation_unit().clone();
    let uses = match &mut compilation {
        CompilationUnit::Program(program) => &mut program.uses,
        CompilationUnit::Unit(unit) => &mut unit.uses,
    };
    if let Some(existing) = uses
        .iter_mut()
        .find(|import| import.unit.parts.join(".").eq_ignore_ascii_case(unit))
    {
        existing.alias = Some(ImportAlias {
            name: alias.to_owned(),
            span: existing.unit.span,
        });
    } else {
        let span = Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
            source_id: 0,
        };
        uses.push(Import {
            unit: QualifiedId {
                parts: unit.split('.').map(str::to_owned).collect(),
                span,
            },
            alias: Some(ImportAlias {
                name: alias.to_owned(),
                span,
            }),
            span,
        });
    }
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
            | SymbolKind::Type
            | SymbolKind::Enum
            | SymbolKind::Function
            | SymbolKind::Procedure
    )
}

//! Lookup of source and generated declarations through the workspace symbol index.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::document::normalized_path;
use crate::documentation::preceding_documentation;
use crate::{
    DocumentSnapshot, DocumentSymbol, LanguageService, SymbolKind, SymbolVisibility,
    WorkspaceSymbolIndex,
};

use super::{ApiEntry, ApiEntryKind, ApiLocation, ApiMember};

/// A declaration that a query may name.
pub(super) struct Candidate {
    pub(super) path: PathBuf,
    pub(super) symbol: DocumentSymbol,
    // Offset of the owning unit or program declaration, whose comment names the handbook page.
    root_offset: usize,
    standard_library: bool,
}

/// Collects unit-level declarations and their record and enum members.
///
/// Routine parameters and locals are never candidates. Standard-library sources contribute
/// only public declarations; project sources contribute private ones too, as hover does.
pub(super) fn candidates(
    service: &LanguageService,
    index: &WorkspaceSymbolIndex,
) -> Vec<Candidate> {
    let standard_library = service
        .editor_api_source_paths()
        .into_iter()
        .chain(service.standard_library_source_paths())
        .map(|path| normalized_path(&path))
        .collect::<HashSet<_>>();
    let mut candidates = Vec::new();
    for (path, symbols) in index.documents() {
        let is_standard_library = standard_library.contains(path);
        for root in symbols.entries() {
            let root_offset = root.full_span.offset();
            let mut push = |symbol: &DocumentSymbol| {
                candidates.push(Candidate {
                    path: path.clone(),
                    symbol: symbol.clone(),
                    root_offset,
                    standard_library: is_standard_library,
                });
            };
            if root.kind == SymbolKind::Unit {
                push(root);
            }
            for declaration in listed(root, is_standard_library) {
                push(declaration);
                if matches!(declaration.kind, SymbolKind::Type | SymbolKind::Enum) {
                    for member in declaration.type_members() {
                        if !is_standard_library || member.visibility == SymbolVisibility::Public {
                            push(member);
                        }
                    }
                }
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.symbol
            .qualified_name
            .to_ascii_lowercase()
            .cmp(&right.symbol.qualified_name.to_ascii_lowercase())
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| {
                left.symbol
                    .full_span
                    .offset()
                    .cmp(&right.symbol.full_span.offset())
            })
    });
    candidates
}

/// Returns entries for candidates whose qualified name equals the query or ends with it.
pub(super) fn matches(
    candidates: &[Candidate],
    snapshots: &HashMap<PathBuf, Arc<DocumentSnapshot>>,
    query: &str,
) -> Vec<ApiEntry> {
    let query = query.to_ascii_lowercase();
    let suffix = format!(".{query}");
    candidates
        .iter()
        .filter(|candidate| {
            let name = candidate.symbol.qualified_name.to_ascii_lowercase();
            name == query || name.ends_with(&suffix)
        })
        .filter_map(|candidate| {
            let snapshot = snapshots.get(&candidate.path)?;
            Some(entry(snapshot, candidate))
        })
        .collect()
}

fn entry(snapshot: &DocumentSnapshot, candidate: &Candidate) -> ApiEntry {
    let source = snapshot.source();
    let symbol = &candidate.symbol;
    let members = if symbol.kind == SymbolKind::Unit {
        listed(symbol, candidate.standard_library)
            .map(|member| ApiMember {
                name: member.name.clone(),
                signature: member.detail.clone(),
                summary: preceding_documentation(source, member.full_span.offset())
                    .and_then(|text| first_line(&text)),
            })
            .collect()
    } else {
        Vec::new()
    };
    ApiEntry {
        name: symbol.qualified_name.clone(),
        kind: ApiEntryKind::Declaration(symbol.kind),
        signature: symbol.detail.clone(),
        documentation: preceding_documentation(source, symbol.full_span.offset()),
        handbook: preceding_documentation(source, candidate.root_offset)
            .and_then(|text| handbook_path(&text)),
        location: location(snapshot, candidate),
        members,
    }
}

// Unit-level declarations of a unit or program, without import aliases.
fn listed(root: &DocumentSymbol, standard_library: bool) -> impl Iterator<Item = &DocumentSymbol> {
    root.children.iter().filter(move |child| {
        matches!(
            child.kind,
            SymbolKind::Constant
                | SymbolKind::Variable
                | SymbolKind::Type
                | SymbolKind::Enum
                | SymbolKind::Function
                | SymbolKind::Procedure
        ) && (!standard_library || child.visibility == SymbolVisibility::Public)
    })
}

fn location(snapshot: &DocumentSnapshot, candidate: &Candidate) -> Option<ApiLocation> {
    let offset = candidate.symbol.selection_span.offset();
    let position = snapshot.line_index().position(offset)?;
    let line_start = snapshot.line_index().line_start(position.line)?;
    let column = snapshot.source().get(line_start..offset)?.chars().count() + 1;
    Some(ApiLocation {
        path: candidate.path.clone(),
        line: position.line + 1,
        column,
    })
}

fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

/// Finds the first repository-relative `docs/pascal/….md` path in a unit comment.
pub(super) fn handbook_path(documentation: &str) -> Option<String> {
    let start = documentation.find("docs/pascal/")?;
    let rest = &documentation[start..];
    let end = rest.find(".md")? + ".md".len();
    let path = &rest[..end];
    path.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.'))
        .then(|| path.to_owned())
}

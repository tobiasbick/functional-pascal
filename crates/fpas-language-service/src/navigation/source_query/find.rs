//! Checked, visibility-filtered declaration discovery with bounded retained candidates.

use super::{
    SourceQueryError, SourceQueryTarget, SourceSession, TargetReference,
    collector::{Collector, DeclarationKey, MatchKey},
    discovery,
};
use crate::{
    CancellationToken, DocumentSnapshot, DocumentSymbol, DocumentSymbols, ScopeKind, SymbolKind,
    SymbolVisibility, identity_path, source_digest,
};
use fpas_diagnostics::FileDiagnostic;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

/// Origin and checking status remain separate from whether a declaration has a source span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationOrigin {
    /// A declaration owned directly by a selected project.
    Owned,
    /// A visible export of an imported dependency.
    Dependency,
    /// A visible declaration in an imported source standard-library unit.
    StandardLibrary,
    /// Source documentation for a compiler-owned intrinsic declaration.
    IntrinsicApi,
    /// A compiler-known operation with no physical declaration.
    Builtin,
    /// An unchecked interactive editor catalogue entry.
    EditorCatalog,
}

/// One bounded search match, including a checked reference only when the full scope permits it.
pub struct SourceCandidate {
    /// Declaration identity and exact source when present.
    pub target: SourceQueryTarget,
    /// Semantic owning project, or no project for intrinsic operations.
    pub owner: Option<String>,
    /// Selected consumer contexts in which this declaration is eligible.
    pub contexts: Vec<String>,
    /// Why this declaration is part of the search scope.
    pub origin: DeclarationOrigin,
    /// False for recovered syntax and editor catalogue entries.
    pub checked: bool,
    /// Existing source documentation, without synthesized prose.
    pub documentation: Option<String>,
    /// Checked callable detail, or source signature for syntax candidates.
    pub detail: String,
}

/// Discovery coverage is independent of the bounded returned collection.
pub struct SourceFindResult {
    /// Deterministically ranked retained declarations.
    pub symbols: Vec<SourceCandidate>,
    /// All matching candidates after complete traversal, including omitted entries.
    pub known_total: Option<usize>,
    /// Original attributed compiler diagnostics.
    pub diagnostics: Vec<FileDiagnostic>,
    /// Missing analysis or output coverage.
    pub reasons: BTreeSet<&'static str>,
}

struct SourceEntry {
    snapshot: Arc<DocumentSnapshot>,
    symbols: DocumentSymbols,
    owner: Option<String>,
    contexts: Vec<String>,
    origin: DeclarationOrigin,
    owned: bool,
    checked: bool,
    intrinsic: Option<super::intrinsic::Catalog>,
}

impl SourceSession {
    /// Searches exact or partial names in an explicit checked manifest context.
    pub fn find_declarations(
        &mut self,
        query: &str,
        exact: bool,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceFindResult, SourceQueryError> {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty()
            || !(1..=10000).contains(&limit)
            || !matches!(
                self.context().kind,
                ScopeKind::Project | ScopeKind::Workspace
            )
            || self.context().policy != super::QueryPolicy::CheckedContext
        {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Declaration search requires nonempty text, a manifest and a limit between 1 and 10000.",
            ));
        }
        let discovery = discovery::checked(self.service(), cancellation)?;
        let mut reasons = discovery.reasons;
        let mut entries = BTreeMap::<(String, PathBuf), SourceEntry>::new();
        let std_sources = self
            .service()
            .standard_library_source_paths()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let api_sources = self.service().editor_api_source_paths();
        let mut imported_units = BTreeSet::new();
        for batch in discovery.batches {
            imported_units.extend(batch.imported.iter().cloned());
            let context = identity_path(batch.project.manifest_path()).map_err(path_error)?;
            for analysis in batch.documents {
                cancellation.check()?;
                let snapshot = analysis.snapshot();
                let owned = batch.project.owns_source(snapshot.path());
                let unit = analysis.symbols().owner();
                let standard = std_sources.contains(snapshot.path());
                if !owned
                    && (!batch.imported.contains(&unit.to_ascii_lowercase())
                        || !standard
                            && !batch.project.all_source_paths().iter().any(|from| {
                                batch.project.owns_source(from)
                                    && batch.project.source_visible_from(
                                        from,
                                        snapshot.path(),
                                        unit,
                                    )
                            }))
                {
                    continue;
                }
                let owner = if owned {
                    context.clone()
                } else if standard {
                    self.context()
                        .standard_library
                        .as_ref()
                        .map(|root| format!("{root}/stdlib.fpasprj"))
                        .unwrap_or_default()
                } else {
                    match batch.project.source_origin(snapshot.path()) {
                        fpas_project::SourceOrigin::Own => context.clone(),
                        fpas_project::SourceOrigin::Library(path) => {
                            identity_path(&path).map_err(path_error)?
                        }
                    }
                };
                let key = (owner.clone(), snapshot.path().into());
                let checked = analysis.semantic().is_some()
                    && !analysis.diagnostics().iter().any(|d| d.is_error());
                if let Some(existing) = entries.get_mut(&key) {
                    // Equal symbol facts may share memberships. Divergent interpretations must not disappear.
                    if existing.symbols != *analysis.symbols() {
                        reasons.insert("ambiguous_context");
                    }
                    existing.checked &= checked;
                    existing.owned |= owned;
                    if owned {
                        existing.origin = DeclarationOrigin::Owned;
                    }
                    existing.contexts.push(context.clone());
                } else {
                    entries.insert(
                        key,
                        SourceEntry {
                            snapshot: Arc::clone(snapshot),
                            symbols: analysis.symbols().clone(),
                            owner: Some(owner),
                            contexts: vec![context.clone()],
                            owned,
                            checked,
                            intrinsic: None,
                            origin: if owned {
                                DeclarationOrigin::Owned
                            } else if standard {
                                DeclarationOrigin::StandardLibrary
                            } else {
                                DeclarationOrigin::Dependency
                            },
                        },
                    );
                }
            }
            for path in &api_sources {
                let snapshot = self.service().snapshot(path)?;
                let symbols = DocumentSymbols::from_editor_snapshot(&snapshot);
                if !batch
                    .imported
                    .contains(&symbols.owner().to_ascii_lowercase())
                {
                    continue;
                }
                let key = (String::new(), path.clone());
                if let Some(existing) = entries.get_mut(&key) {
                    existing.contexts.push(context.clone());
                } else {
                    entries.insert(
                        key,
                        SourceEntry {
                            intrinsic: Some(super::intrinsic::Catalog::new(symbols.owner())),
                            snapshot,
                            symbols,
                            owner: None,
                            contexts: vec![context.clone()],
                            owned: false,
                            checked: true,
                            origin: DeclarationOrigin::IntrinsicApi,
                        },
                    );
                }
            }
        }
        let mut collector = Collector::new(limit);
        for entry in entries.values_mut() {
            cancellation.check()?;
            entry.contexts.sort();
            entry.contexts.dedup();
            let file = identity_path(entry.snapshot.path()).map_err(path_error)?;
            collect_symbols(
                entry.symbols.entries(),
                entry,
                &query,
                exact,
                &file,
                &mut collector,
                false,
            );
        }
        for unit in imported_units {
            for registered in fpas_sema::intrinsic_std_symbols(&unit) {
                cancellation.check()?;
                if entries.values().any(|entry| {
                    entry.origin == DeclarationOrigin::IntrinsicApi
                        && contains(entry.symbols.entries(), &registered.qualified_name)
                }) {
                    continue;
                }
                let name = registered
                    .qualified_name
                    .rsplit('.')
                    .next()
                    .unwrap_or(&registered.qualified_name);
                if let Some(key) = MatchKey::new(
                    &query,
                    exact,
                    DeclarationKey {
                        name,
                        qualified: &registered.qualified_name,
                        owner: "",
                        file: "",
                        start: 0,
                        kind: "builtin",
                    },
                ) {
                    collector.offer(key, || SourceCandidate {
                        target: SourceQueryTarget {
                            name: name.into(),
                            qualified_name: registered.qualified_name.clone(),
                            kind: Some(super::intrinsic::kind(registered.kind)),
                            symbol: None,
                            snapshot: None,
                            target_ref: None,
                            reference_unavailable: Some("no_source"),
                        },
                        owner: None,
                        contexts: Vec::new(),
                        origin: DeclarationOrigin::Builtin,
                        checked: true,
                        documentation: None,
                        detail: registered.ty.to_string(),
                    });
                }
            }
        }
        for &name in super::BUILTIN_TYPES {
            if let Some(key) = MatchKey::new(
                &query,
                exact,
                DeclarationKey {
                    name,
                    qualified: name,
                    owner: "",
                    file: "",
                    start: 0,
                    kind: "builtin_type",
                },
            ) {
                collector.offer(key, || SourceCandidate {
                    target: SourceQueryTarget {
                        name: name.into(),
                        qualified_name: name.into(),
                        kind: Some(SymbolKind::Type),
                        symbol: None,
                        snapshot: None,
                        target_ref: None,
                        reference_unavailable: Some("no_source"),
                    },
                    owner: None,
                    contexts: Vec::new(),
                    origin: DeclarationOrigin::Builtin,
                    checked: true,
                    documentation: None,
                    detail: name.into(),
                });
            }
        }
        for native in fpas_sema::native_operations() {
            cancellation.check()?;
            let qualified = crate::api_lookup::native_qualified_name(native);
            if let Some(key) = MatchKey::new(
                &query,
                exact,
                DeclarationKey {
                    name: native.name,
                    qualified: &qualified,
                    owner: "",
                    file: "",
                    start: 0,
                    kind: "builtin",
                },
            ) {
                collector.offer(key, || SourceCandidate {
                    target: SourceQueryTarget {
                        name: native.name.into(),
                        qualified_name: qualified,
                        kind: Some(SymbolKind::Method),
                        symbol: None,
                        snapshot: None,
                        target_ref: None,
                        reference_unavailable: Some("no_source"),
                    },
                    owner: None,
                    contexts: Vec::new(),
                    origin: DeclarationOrigin::Builtin,
                    checked: true,
                    documentation: Some(native.documentation.into()),
                    detail: crate::intellisense::native::signature(native, None).label,
                });
            }
        }
        let (mut symbols, total) = collector.finish();
        if !self.verify()? {
            reasons.insert("source_changed");
        }
        let complete = reasons.is_empty();
        for candidate in &mut symbols {
            if complete && candidate.checked {
                if let Some((snapshot, symbol)) = candidate
                    .target
                    .snapshot
                    .as_ref()
                    .zip(candidate.target.symbol.as_ref())
                {
                    if self
                        .service()
                        .workspace()
                        .projects()
                        .iter()
                        .filter(|p| p.owns_source(snapshot.path()))
                        .count()
                        > 1
                    {
                        candidate.target.reference_unavailable = Some("ambiguous_context");
                        continue;
                    }
                    candidate.target.target_ref = Some(TargetReference {
                        version: 1,
                        context: self.context().clone(),
                        snapshot_id: self.snapshot().id.clone(),
                        analysis_identity: self.snapshot().analysis_identity.clone(),
                        owner: candidate.owner.clone(),
                        kind: symbol.kind,
                        qualified_name: symbol.qualified_name.clone(),
                        file: identity_path(snapshot.path()).map_err(path_error)?,
                        start: symbol.selection_span.offset(),
                        length: symbol.selection_span.length(),
                        file_sha256: source_digest(snapshot.source().as_bytes()),
                    });
                    candidate.target.reference_unavailable = None;
                }
            } else {
                candidate.target.reference_unavailable = Some("analysis_incomplete");
            }
        }
        if total > symbols.len() {
            reasons.insert("result_limit");
        }
        Ok(SourceFindResult {
            symbols,
            known_total: complete.then_some(total),
            diagnostics: discovery.diagnostics,
            reasons,
        })
    }
}

fn collect_symbols(
    symbols: &[DocumentSymbol],
    entry: &SourceEntry,
    query: &str,
    exact: bool,
    file: &str,
    collector: &mut Collector<SourceCandidate>,
    exported_parent: bool,
) {
    for symbol in symbols {
        if entry
            .intrinsic
            .as_ref()
            .is_some_and(|catalog| !catalog.known(symbol))
        {
            continue;
        }
        let root = symbol.kind == SymbolKind::Unit;
        let member = matches!(
            symbol.kind,
            SymbolKind::Field | SymbolKind::Method | SymbolKind::EnumMember
        );
        let exported = symbol.visibility == SymbolVisibility::Public || exported_parent && member;
        if !entry.owned && !exported {
            continue;
        }
        // Import aliases have a file-local meaning and are selected only by position/reference.
        if symbol.kind != SymbolKind::ImportAlias
            && let Some(key) = MatchKey::new(
                query,
                exact,
                DeclarationKey {
                    name: &symbol.name,
                    qualified: &symbol.qualified_name,
                    owner: entry.owner.as_deref().unwrap_or(""),
                    file,
                    start: symbol.selection_span.offset(),
                    kind: &format!("{:?}", symbol.kind),
                },
            )
        {
            collector.offer(key, || {
                let mut shallow = symbol.clone();
                shallow.children.clear();
                SourceCandidate {
                    target: SourceQueryTarget {
                        name: symbol.name.clone(),
                        qualified_name: symbol.qualified_name.clone(),
                        kind: Some(symbol.kind),
                        symbol: Some(shallow),
                        snapshot: Some(Arc::clone(&entry.snapshot)),
                        target_ref: None,
                        reference_unavailable: Some("analysis_incomplete"),
                    },
                    owner: entry.owner.clone(),
                    contexts: entry.contexts.clone(),
                    origin: entry.origin,
                    checked: entry.checked,
                    detail: symbol.detail.clone(),
                    documentation: crate::documentation::preceding_documentation(
                        entry.snapshot.source(),
                        symbol.full_span.offset(),
                    ),
                }
            });
        }
        if entry.owned
            || root
            || matches!(
                symbol.kind,
                SymbolKind::Type | SymbolKind::Enum | SymbolKind::EnumMember
            )
        {
            collect_symbols(
                &symbol.children,
                entry,
                query,
                exact,
                file,
                collector,
                !root && exported,
            );
        }
    }
}

fn path_error(error: std::io::Error) -> SourceQueryError {
    SourceQueryError::new("unsupported_path_identity", error.to_string())
}

fn contains(symbols: &[DocumentSymbol], name: &str) -> bool {
    symbols
        .iter()
        .any(|s| s.qualified_name.eq_ignore_ascii_case(name) || contains(&s.children, name))
}

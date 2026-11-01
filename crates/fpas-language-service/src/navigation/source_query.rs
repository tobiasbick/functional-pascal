//! Strict saved-source questions with target identity and explicit semantic coverage.

mod calls;
pub(crate) use calls::visit_inventory as visit_call_inventory;
mod catalog;
pub use calls::{
    CallArgument, CallArgumentProjection, CallCollection, CallCollectionCoverage,
    CallInventoryCoverage, CallRelation, CallRelationKind, CallRelationProjection, CallRelations,
    CallRelationsProjection, CallableIdentity, CallableKind, CallableProjection, CallableReference,
    InvocationKind, SourceCallable,
};
pub(super) mod collector;
pub(super) mod discovery;
mod find;
mod inspect;
pub(super) mod intrinsic;
pub use inspect::{
    INSPECT_EXCERPT_BYTES, INSPECT_SUPPORTING_TYPES, Inspection, InspectionExcerptProjection,
    InspectionOmission, InspectionProjection, InspectionSection, InspectionSectionKind,
    InspectionSectionProjection, SourceExcerpt,
};
mod scope;
mod selection;
pub use find::{DeclarationOrigin, SourceCandidate, SourceFindResult};
pub use scope::{QueryPolicy, ScopeContext, ScopeKind, ScopeSnapshot, SourceSession};
mod reference;
pub use reference::{SourceQueryError, TargetReference};

use super::{ExpressionType, ReferenceLocation, resolve_target};
use crate::{CancellationToken, DocumentSnapshot, DocumentSymbol, LanguageService};
use fpas_diagnostics::FileDiagnostic;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

/// Operation performed by a saved-source query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceQueryOperation {
    /// Proven incoming calls and related callable uses in the checked context.
    Callers,
    /// Proven outgoing calls and unresolved sites in the selected source body.
    Callees,
    /// Declaration, real implementation and directly named supporting types.
    Inspect,
    /// Resolved uses, excluding the defining declaration.
    #[serde(rename = "refs")]
    References,
    /// Defining identifier.
    Definition,
    /// Checked type of the selected construct.
    Type,
}

/// The resolved declaration or source-less built-in selected by a query.
pub struct SourceQueryTarget {
    /// Case-preserving declaration name or canonical built-in API name.
    pub name: String,
    /// Owner-qualified source identity.
    pub qualified_name: String,
    /// Known declaration category, including compiler-owned declarations without source.
    pub kind: Option<crate::SymbolKind>,
    /// Source symbol when backed by a physical declaration.
    pub symbol: Option<DocumentSymbol>,
    /// Immutable declaration source; absent for built-ins.
    pub snapshot: Option<Arc<DocumentSnapshot>>,
    /// Portable identity, issued only after complete scope and declaration verification.
    pub target_ref: Option<TargetReference>,
    /// Why a portable reference is unavailable for this result.
    pub reference_unavailable: Option<&'static str>,
}

/// Semantic result; absence of uses is independent from target resolution and coverage.
pub struct SourceQueryResult {
    /// Call relationships, separate from ordinary identifier references.
    pub calls: Option<CallRelations>,
    /// Selected declaration identity, if resolved.
    pub target: Option<SourceQueryTarget>,
    /// Proven source uses or definition.
    pub matches: Vec<ReferenceLocation>,
    /// Checked type when requested and available.
    pub expression_type: Option<ExpressionType>,
    /// Compiler-known value type when the declaration has no physical source expression.
    pub declaration_type: Option<DeclarationType>,
    /// Bounded inspection context when requested.
    pub inspection: Option<Inspection>,
    /// Original attributed FP diagnostics from the selected context.
    pub diagnostics: Vec<FileDiagnostic>,
    /// Reasons why semantic coverage is incomplete.
    pub reasons: BTreeSet<&'static str>,
    /// Bounded name candidates when selection is ambiguous or incomplete.
    pub candidates: Vec<SourceCandidate>,
    /// Exact candidate count only after complete eligible discovery.
    pub candidate_total: Option<usize>,
    /// Complete selection failure, distinct from missing analysis coverage.
    pub selection_error: Option<&'static str>,
}

/// A checked source-less value type, without an invented source range.
pub struct DeclarationType {
    /// Compiler type spelling.
    pub display: String,
    /// Stable semantic type category.
    pub kind: &'static str,
}

impl LanguageService {
    /// Queries a loaded context without guessing completeness from an empty result.
    /// `limit` bounds reference retention during traversal.
    pub fn source_query(
        &mut self,
        path: &Path,
        offset: usize,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceQueryResult, SourceQueryError> {
        let mut result = SourceQueryResult {
            calls: None,
            target: None,
            matches: Vec::new(),
            expression_type: None,
            declaration_type: None,
            inspection: None,
            diagnostics: Vec::new(),
            reasons: BTreeSet::new(),
            candidates: Vec::new(),
            candidate_total: None,
            selection_error: None,
        };
        let mut paths = self
            .workspace()
            .projects()
            .iter()
            .flat_map(|p| p.all_source_paths())
            .collect::<BTreeSet<_>>();
        paths.insert(crate::document::normalized_path(path));
        let discovery = discovery::checked(self, cancellation)?;
        result.reasons.extend(discovery.reasons);
        result.diagnostics.extend(discovery.diagnostics);
        let mut analyses = BTreeMap::new();
        for batch in discovery.batches {
            for analysis in batch.documents {
                retain_analysis(&mut analyses, analysis);
            }
        }
        let additional = paths
            .iter()
            .filter(|p| !analyses.contains_key(*p))
            .cloned()
            .collect::<Vec<_>>();
        for source in &additional {
            cancellation.check()?;
            match self.source_analyses(source) {
                Ok(documents) => {
                    for analysis in documents {
                        retain_analysis(&mut analyses, analysis);
                    }
                }
                Err(failure) => {
                    result.reasons.insert("analysis_unavailable");
                    result.diagnostics.extend(failure.diagnostics());
                    for analysis in self.source_syntax_analyses(source)? {
                        retain_analysis(&mut analyses, analysis);
                    }
                }
            }
        }
        let mut relevant = paths.clone();
        let mut pending = paths.iter().cloned().collect::<Vec<_>>();
        let units = analyses
            .iter()
            .filter_map(|(path, analysis)| {
                if let fpas_parser::CompilationUnit::Unit(unit) =
                    analysis.snapshot().compilation_unit()
                {
                    Some((unit.name.parts.join(".").to_ascii_lowercase(), path.clone()))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<_, _>>();
        while let Some(path) = pending.pop() {
            cancellation.check()?;
            let Some(analysis) = analyses.get(&path) else {
                continue;
            };
            let imports = match analysis.snapshot().compilation_unit() {
                fpas_parser::CompilationUnit::Unit(unit) => &unit.uses,
                fpas_parser::CompilationUnit::Program(program) => &program.uses,
            };
            for import in imports {
                if let Some(path) = units.get(&import.unit.parts.join(".").to_ascii_lowercase())
                    && relevant.insert(path.clone())
                {
                    pending.push(path.clone());
                }
            }
        }
        for (source, analysis) in analyses.iter().filter(|(path, _)| relevant.contains(*path)) {
            for diagnostic in analysis.diagnostics() {
                if diagnostic.is_error() {
                    result.reasons.insert("analysis_errors");
                }
                result.diagnostics.push(
                    FileDiagnostic::new(diagnostic.clone(), Some(source.clone()))
                        .with_source(analysis.snapshot().source()),
                );
            }
        }
        let context = if operation == SourceQueryOperation::References {
            self.reference_navigation_context(path, offset, cancellation)?
        } else {
            self.navigation_context(path)?
        };
        if let Some(index) = context.target_index {
            if let Some(target) = resolve_target(&context.documents, index, offset) {
                let document = &context.documents[target.document_index];
                result.target = Some(SourceQueryTarget {
                    name: target.symbol.name.clone(),
                    qualified_name: target.symbol.qualified_name.clone(),
                    kind: Some(target.symbol.kind),
                    symbol: Some(target.symbol.clone()),
                    snapshot: Some(Arc::clone(&document.snapshot)),
                    target_ref: None,
                    reference_unavailable: Some("scope_unverified"),
                });
                if operation == SourceQueryOperation::References {
                    // Standard-library bodies participate only when selected themselves.
                    let declaration = document.path.clone();
                    let mut documents = context.documents;
                    documents.retain(|d| {
                        paths.contains(&d.path) || d.path == declaration || d.is_editor_api
                    });
                    let selected = documents
                        .iter()
                        .position(|d| d.path == context.snapshot.path());
                    if let Some(target) =
                        selected.and_then(|i| resolve_target(&documents, i, offset))
                    {
                        let (matches, complete) = super::references::find_references_bounded(
                            &documents,
                            &target,
                            false,
                            limit,
                            cancellation,
                        )?;
                        result.matches = matches;
                        if !complete {
                            result.reasons.insert("result_limit");
                        }
                    } else {
                        result.reasons.insert("analysis_unavailable");
                    }
                } else if matches!(
                    operation,
                    SourceQueryOperation::Definition | SourceQueryOperation::Inspect
                ) {
                    result.matches.push(ReferenceLocation {
                        path: document.path.clone(),
                        span: target.symbol.selection_span,
                        is_declaration: true,
                        snapshot: Arc::clone(&document.snapshot),
                    });
                }
            } else if let Some((native, _)) =
                crate::intellisense::native::native_at(&context.documents, index, offset)
            {
                result.target = Some(SourceQueryTarget {
                    name: native.name.into(),
                    qualified_name: crate::api_lookup::native_qualified_name(native),
                    kind: Some(crate::SymbolKind::Method),
                    symbol: None,
                    snapshot: None,
                    target_ref: None,
                    reference_unavailable: Some("no_source"),
                });
                if operation == SourceQueryOperation::References {
                    result.reasons.insert("unsupported_target");
                }
            } else if let Some(name) = builtin_at(&context.documents[index], offset) {
                result.target = Some(SourceQueryTarget {
                    name: name.into(),
                    qualified_name: name.into(),
                    kind: Some(crate::SymbolKind::Type),
                    symbol: None,
                    snapshot: None,
                    target_ref: None,
                    reference_unavailable: Some("no_source"),
                });
                if operation == SourceQueryOperation::References {
                    result.reasons.insert("unsupported_target");
                }
            }
        }
        if operation == SourceQueryOperation::Type {
            let (selection, unavailable) = analyses
                .get(&crate::document::normalized_path(path))
                .map(|analysis| super::expression_type::checked_at(analysis, offset))
                .unwrap_or((None, true));
            result.expression_type = selection;
            if unavailable {
                result.reasons.insert("type_unavailable");
            }
            if let Some(expression) = &mut result.expression_type {
                expression.declaration =
                    nominal_declaration(&self.navigation_context(path)?, &expression.ty);
            }
            if let Some(target) = &result.target
                && let Some((snapshot, symbol)) =
                    target.snapshot.as_ref().zip(target.symbol.as_ref())
                && self.is_editor_api_source(snapshot.path())
                && let Some(registered) = intrinsic::lookup(&target.qualified_name)
            {
                result.reasons.remove("type_unavailable");
                result.expression_type = if matches!(
                    symbol.kind,
                    crate::SymbolKind::Type | crate::SymbolKind::Enum
                ) {
                    None
                } else {
                    Some(ExpressionType {
                        type_name: registered.ty.to_string(),
                        kind: registered.ty.category(),
                        span: symbol.selection_span,
                        declaration: None,
                        ty: registered.ty,
                    })
                };
            }
            if result
                .target
                .as_ref()
                .and_then(|t| t.symbol.as_ref())
                .is_some_and(|s| {
                    matches!(
                        s.kind,
                        crate::SymbolKind::Unit | crate::SymbolKind::ImportAlias
                    )
                })
            {
                result.expression_type = None;
                result.reasons.remove("type_unavailable");
            }
        }
        if operation == SourceQueryOperation::Inspect
            && let Some(target) = &result.target
        {
            let mut inspection = self.inspect_target(target, cancellation)?;
            if !inspection.analysis_complete {
                result.reasons.insert("inspection_context_unavailable");
            }
            if !result.reasons.is_empty() {
                for section in &mut inspection.sections {
                    section.checked = false;
                }
            }
            result.inspection = Some(inspection);
        }
        result.diagnostics.sort_by(|a, b| {
            a.path.cmp(&b.path).then_with(|| {
                a.diagnostic
                    .span
                    .map(|s| s.offset())
                    .cmp(&b.diagnostic.span.map(|s| s.offset()))
            })
        });
        result.diagnostics.dedup();
        Ok(result)
    }
}

fn retain_analysis(
    analyses: &mut BTreeMap<std::path::PathBuf, Arc<crate::DocumentAnalysis>>,
    analysis: Arc<crate::DocumentAnalysis>,
) {
    let path = analysis.snapshot().path();
    if analyses
        .get(path)
        .is_none_or(|existing| existing.semantic().is_none())
    {
        analyses.insert(path.to_path_buf(), analysis);
    }
}

fn nominal_declaration(
    context: &super::service::NavigationContext,
    ty: &fpas_sema::Ty,
) -> Option<ReferenceLocation> {
    use fpas_sema::Ty;
    let (name, owner) = match ty {
        Ty::Record(record) => (record.name.as_str(), record.owner_unit.as_deref()),
        Ty::Distinct(distinct) => (distinct.name.as_str(), distinct.owner_unit.as_deref()),
        Ty::Enum(enumeration) => (enumeration.name.as_str(), None),
        Ty::Subrange(range) => (range.name.as_str(), None),
        _ => return None,
    };
    let selected = context.target_index?;
    let name = if name.contains('.') {
        name.to_owned()
    } else {
        owner.map_or_else(|| name.to_owned(), |owner| format!("{owner}.{name}"))
    };
    let (index, symbol) = super::find_type(&context.documents, selected, selected, &name)?;
    let document = &context.documents[index];
    Some(ReferenceLocation {
        path: document.path.clone(),
        span: symbol.selection_span,
        is_declaration: true,
        snapshot: Arc::clone(&document.snapshot),
    })
}

const BUILTIN_TYPES: &[&str] = &[
    "integer", "real", "boolean", "string", "array", "dict", "channel", "task", "Option", "Result",
];

fn builtin_at(document: &super::NavigationDocument, offset: usize) -> Option<&'static str> {
    use fpas_lexer::Token;
    let token = document
        .tokens
        .iter()
        .find(|t| t.span.offset <= offset && offset < t.span.offset + t.span.length)?;
    match &token.token {
        Token::Ident(name) => BUILTIN_TYPES
            .iter()
            .copied()
            .find(|s| s.eq_ignore_ascii_case(name)),
        Token::Array => Some("array"),
        Token::Dict => Some("dict"),
        Token::Channel => Some("channel"),
        Token::Task => Some("task"),
        Token::OptionKw | Token::Some | Token::None => Some("Option"),
        Token::Result | Token::Ok | Token::Error => Some("Result"),
        _ => None,
    }
}

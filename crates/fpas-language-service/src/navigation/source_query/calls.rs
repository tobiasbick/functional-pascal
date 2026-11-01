//! Streaming checked call inventory shared by queries, hierarchy and signature changes.

mod index;
mod model;
mod projection;
mod resolution;
pub use projection::{
    CallArgumentProjection, CallCollectionCoverage, CallRelationProjection,
    CallRelationsProjection, CallableProjection,
};
mod hierarchy;
mod walk;
pub use hierarchy::CallableReference;
pub use model::*;

use super::{
    SourceQueryError, SourceQueryOperation, SourceQueryTarget, SourceSession, collector::Collector,
    discovery,
};
use crate::{CancellationToken, identity_path};
use fpas_diagnostics::FileDiagnostic;
use std::collections::BTreeSet;

type Order = (
    String,
    usize,
    usize,
    CallRelationKind,
    Option<CallableIdentity>,
    String,
    CallableIdentity,
);

struct Collections {
    direct: Collector<CallRelation, Order>,
    values: Collector<CallRelation, Order>,
    callbacks: Collector<CallRelation, Order>,
    dynamic: Collector<CallRelation, Order>,
}
impl Collections {
    fn new(limit: usize) -> Self {
        Self {
            direct: Collector::new(limit),
            values: Collector::new(limit),
            callbacks: Collector::new(limit),
            dynamic: Collector::new(limit),
        }
    }
    fn offer(&mut self, relation: CallRelation) {
        let key = (
            relation
                .containing
                .identity
                .file
                .clone()
                .unwrap_or_default(),
            relation.span.offset(),
            relation.span.length(),
            relation.kind,
            relation.target.as_ref().map(|t| t.identity.clone()),
            relation.context.clone(),
            relation.containing.identity.clone(),
        );
        let collection = match relation.kind {
            CallRelationKind::DirectCall => &mut self.direct,
            CallRelationKind::CallableValue => &mut self.values,
            CallRelationKind::CallbackArgument => &mut self.callbacks,
            CallRelationKind::DynamicCall => &mut self.dynamic,
        };
        collection.offer(key, || relation);
    }
}

/// Complete traversal coverage, separate from the caller's retained relationship budget.
pub struct CallInventoryCoverage {
    /// Original attributed diagnostics for every eligible consumer context.
    pub diagnostics: Vec<FileDiagnostic>,
    /// Missing checked source facts or excluded bodies.
    pub reasons: BTreeSet<&'static str>,
}

impl SourceSession {
    /// Visits every eligible checked relationship without a presentation limit.
    /// The consumer must enforce its own retained-data budget and may stop with an error.
    pub fn visit_call_relations(
        &mut self,
        selected_body: Option<&std::path::Path>,
        cancellation: &CancellationToken,
        mut visit: impl FnMut(CallRelation) -> Result<(), SourceQueryError>,
    ) -> Result<CallInventoryCoverage, SourceQueryError> {
        self.visit_call_inventory(selected_body, cancellation, &mut visit, &mut |_| {})
    }

    fn visit_call_inventory(
        &mut self,
        selected_body: Option<&std::path::Path>,
        cancellation: &CancellationToken,
        visit: &mut dyn FnMut(CallRelation) -> Result<(), SourceQueryError>,
        owner: &mut dyn FnMut(&std::sync::Arc<SourceCallable>),
    ) -> Result<CallInventoryCoverage, SourceQueryError> {
        if self.context().policy != crate::QueryPolicy::CheckedContext {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Call relationships require a checked context.",
            ));
        }
        let context = self.context().clone();
        let mut coverage = visit_inventory(
            self.service(),
            &context,
            selected_body,
            cancellation,
            visit,
            owner,
        )?;
        if !self.verify()? {
            coverage.reasons.insert("source_changed");
        }
        Ok(coverage)
    }

    pub(super) fn call_relations(
        &mut self,
        target: &SourceQueryTarget,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<(CallRelations, Vec<FileDiagnostic>), SourceQueryError> {
        self.collect_target_relations(target, operation, limit, cancellation)
    }
}

/// Traverses the same checked inventory in an isolated edit overlay.
pub(crate) fn visit_inventory(
    service: &mut crate::LanguageService,
    scope: &crate::ScopeContext,
    selected_body: Option<&std::path::Path>,
    cancellation: &CancellationToken,
    visit: &mut dyn FnMut(CallRelation) -> Result<(), SourceQueryError>,
    owner: &mut dyn FnMut(&std::sync::Arc<SourceCallable>),
) -> Result<CallInventoryCoverage, SourceQueryError> {
    let discovery = discovery::checked(service, cancellation)?;
    let loose = discovery.batches.is_empty() && scope.kind == crate::ScopeKind::Loose;
    let mut coverage = CallInventoryCoverage {
        diagnostics: discovery.diagnostics,
        reasons: discovery.reasons,
    };
    let standard = service
        .standard_library_source_paths()
        .into_iter()
        .collect::<BTreeSet<_>>();
    if loose {
        let path = std::path::PathBuf::from(&scope.input);
        let documents = match service.source_analyses(&path) {
            Ok(documents) => documents,
            Err(error) => {
                coverage.reasons.insert("analysis_unavailable");
                coverage.diagnostics.extend(error.diagnostics());
                service.source_syntax_analyses(&path)?
            }
        };
        let index = index::Index::new(&documents, &BTreeSet::from([path.clone()]), &standard)?;
        for analysis in &documents {
            if analysis.snapshot().path() == path
                || selected_body == Some(analysis.snapshot().path())
            {
                for diagnostic in analysis.diagnostics() {
                    if diagnostic.is_error() {
                        coverage.reasons.insert("analysis_errors");
                    }
                    coverage.diagnostics.push(
                        FileDiagnostic::new(
                            diagnostic.clone(),
                            Some(analysis.snapshot().path().into()),
                        )
                        .with_source(analysis.snapshot().source()),
                    );
                }
                coverage.reasons.extend(walk::Walker::document(
                    analysis,
                    &index,
                    &scope.input,
                    cancellation,
                    visit,
                    owner,
                )?);
            }
        }
    }
    for batch in discovery.batches {
        cancellation.check()?;
        let owned = batch
            .project
            .all_source_paths()
            .into_iter()
            .filter(|path| batch.project.owns_source(path))
            .collect::<BTreeSet<_>>();
        let eligible = batch
            .project
            .all_source_paths()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let context = identity_path(batch.project.manifest_path())
            .map_err(|e| SourceQueryError::new("unsupported_path_identity", e.to_string()))?;
        let index = index::Index::new(&batch.documents, &owned, &standard)?;
        for analysis in &batch.documents {
            let path = analysis.snapshot().path();
            if eligible.contains(path) || selected_body == Some(path) {
                coverage.reasons.extend(walk::Walker::document(
                    analysis,
                    &index,
                    &context,
                    cancellation,
                    visit,
                    owner,
                )?);
            }
        }
    }
    Ok(coverage)
}

impl SourceSession {
    fn collect_target_relations(
        &mut self,
        target: &SourceQueryTarget,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<(CallRelations, Vec<FileDiagnostic>), SourceQueryError> {
        let incoming = operation == SourceQueryOperation::Callers;
        let supported = matches!(
            target.kind,
            Some(
                crate::SymbolKind::Function
                    | crate::SymbolKind::Procedure
                    | crate::SymbolKind::Method
                    | crate::SymbolKind::Program
                    | crate::SymbolKind::Unit
                    | crate::SymbolKind::Field
            )
        );
        let mut reasons = BTreeSet::new();
        if !supported {
            reasons.insert("unsupported_target");
        }
        if !incoming
            && (target.snapshot.is_none()
                || super::intrinsic::lookup(&target.qualified_name).is_some())
        {
            reasons.insert("body_unavailable");
        }
        self.collect_relations(
            target.snapshot.as_ref().map(|s| s.path()),
            incoming,
            limit,
            reasons,
            |callable| matches_target(callable, target),
            cancellation,
        )
    }

    fn collect_relations(
        &mut self,
        selected_body: Option<&std::path::Path>,
        incoming: bool,
        limit: usize,
        mut reasons: BTreeSet<&'static str>,
        matches: impl Fn(&SourceCallable) -> bool,
        cancellation: &CancellationToken,
    ) -> Result<(CallRelations, Vec<FileDiagnostic>), SourceQueryError> {
        let mut collections = Collections::new(limit);
        let mut dynamic = false;
        let mut unavailable = false;
        let coverage = self.visit_call_relations(selected_body, cancellation, |relation| {
            let selected_owner = matches(&relation.containing);
            let selected_target = relation.target.as_ref().is_some_and(|t| matches(t));
            let relevant = if incoming {
                selected_target || relation.target.is_none()
            } else {
                selected_owner
            };
            if relevant {
                dynamic |= relation.kind == CallRelationKind::DynamicCall;
                unavailable |= relation.unresolved_reason == Some("checked_target_unavailable")
                    || relation.unresolved_reason == Some("declaration_unavailable");
                collections.offer(relation);
            }
            Ok(())
        })?;
        reasons.extend(coverage.reasons);
        if unavailable {
            reasons.insert("checked_target_unavailable");
        }
        let complete = reasons.is_empty();
        let direct_calls = finish(collections.direct, complete);
        let callable_values = finish(collections.values, complete);
        let callback_arguments = finish(collections.callbacks, complete);
        let dynamic_calls = finish(collections.dynamic, complete);
        if [
            &direct_calls,
            &callable_values,
            &callback_arguments,
            &dynamic_calls,
        ]
        .iter()
        .any(|c| !c.items_complete)
        {
            reasons.insert("result_limit");
        }
        if dynamic {
            reasons.insert("dynamic_targets");
        }
        Ok((
            CallRelations {
                direct_calls,
                callable_values,
                callback_arguments,
                dynamic_calls,
                direct_edges_complete: complete,
                runtime_targets_complete: complete && !dynamic,
                reasons,
            },
            coverage.diagnostics,
        ))
    }
}

fn finish(collector: Collector<CallRelation, Order>, complete: bool) -> CallCollection {
    let (items, total) = collector.finish();
    let items_complete = items.len() == total;
    CallCollection {
        items,
        known_total: complete.then_some(total),
        items_complete,
    }
}

fn matches_target(callable: &SourceCallable, target: &SourceQueryTarget) -> bool {
    if callable.identity.kind == CallableKind::Builtin {
        return callable
            .identity
            .qualified_name
            .eq_ignore_ascii_case(&target.qualified_name);
    }
    if let Some((snapshot, symbol)) = target.snapshot.as_ref().zip(target.symbol.as_ref()) {
        callable
            .snapshot
            .as_ref()
            .is_some_and(|s| s.path() == snapshot.path())
            && callable.identity.start == Some(symbol.full_span.offset())
    } else {
        callable.identity.kind == CallableKind::Builtin
            && callable
                .identity
                .qualified_name
                .eq_ignore_ascii_case(&target.qualified_name)
    }
}

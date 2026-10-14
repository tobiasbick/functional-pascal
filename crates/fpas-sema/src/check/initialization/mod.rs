//! Declaration-resolved initialization dependencies and conservative availability checks.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

mod calls;
mod collection;
mod expressions;
mod external_calls;
mod statements;
mod walker;
mod workers;

use crate::check::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_lexer::Span;
use fpas_parser::{Decl, Designator, DesignatorPart, Stmt};
use std::collections::{BTreeMap, HashMap};

type BindingId = (u32, usize);

fn identity(span: Span) -> BindingId {
    (span.source_id, span.offset)
}

#[derive(Clone)]
struct Reference {
    parts: usize,
    declaration: Option<BindingId>,
    name: String,
    kind: SymbolKind,
    ty: Ty,
}

/// Semantic references and dependency summaries, retained across checking phases.
#[derive(Default)]
pub(crate) struct InitializationState {
    references: HashMap<BindingId, Reference>,
    /// Dependency summaries imported from independently compiled units.
    pub(crate) imported: HashMap<String, fpas_unit::interface::RoutineInitialization>,
    /// Publicly persistent summaries of local named routines.
    pub(crate) exported: BTreeMap<String, fpas_unit::interface::RoutineInitialization>,
}

impl InitializationState {
    /// Expose compiler-selected unqualified bindings to editor queries.
    pub(crate) fn binding_references(&self) -> HashMap<BindingId, BindingId> {
        self.references
            .iter()
            .filter_map(|(&site, reference)| {
                (reference.parts == 1)
                    .then_some(reference.declaration)
                    .flatten()
                    .map(|declaration| (site, declaration))
            })
            .collect()
    }
}

impl Checker {
    /// Retain the actual semantic binding rather than re-resolving names in the proof pass.
    pub(crate) fn record_initialization_reference(&mut self, designator: &Designator) {
        let mut name = String::new();
        let mut selected = None;
        for (index, part) in designator.parts.iter().enumerate() {
            let DesignatorPart::Ident(part, _) = part else {
                break;
            };
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(part);
            if let Some((_, symbol, declaration)) =
                self.scopes.lookup_with_scope_and_declaration(&name)
            {
                let canonical = self.scopes.imports.canonical_name(&name);
                selected = Some(Reference {
                    parts: index + 1,
                    declaration: declaration.map(identity),
                    name: canonical,
                    kind: symbol.kind,
                    ty: symbol.ty.clone(),
                });
            }
        }
        if let Some(reference) = selected {
            let entry = self
                .initialization
                .references
                .entry(identity(designator.span));
            entry
                .and_modify(|existing| {
                    if existing.parts <= reference.parts {
                        *existing = reference.clone();
                    }
                })
                .or_insert(reference);
        }
    }

    /// Infer recursive dependency summaries and validate every actual initialization/use point.
    pub(crate) fn check_initialization(&mut self, declarations: &[Decl], body: Option<&[Stmt]>) {
        if !self.errors.is_empty() {
            return;
        }
        let model = collection::Model::collect(declarations, body, self);
        let mut summaries = HashMap::new();
        for &key in model.routines.keys() {
            summaries.insert(key, walker::Summary::default());
        }
        loop {
            let mut changed = false;
            for (&key, routine) in &model.routines {
                let mut walk = walker::Walker::new(self, &model, &summaries, key, false);
                walk.routine(routine);
                if summaries.get(&key) != Some(&walk.summary) {
                    summaries.insert(key, walk.summary);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut failures = Vec::new();
        let mut task_failures = Vec::new();
        for (&key, routine) in &model.routines {
            let mut walk = walker::Walker::new(self, &model, &summaries, key, true);
            walk.routine(routine);
            failures.extend(walk.errors);
            task_failures.extend(walk.task_errors);
        }
        let mut walk = walker::Walker::new(self, &model, &summaries, collection::Owner::Root, true);
        walk.initializers(declarations);
        if let Some(body) = body {
            walk.statements(body);
        }
        failures.extend(walk.errors);
        task_failures.extend(walk.task_errors);
        let exported = model
            .routines
            .iter()
            .filter_map(|(&key, routine)| {
                let name = routine.export_name.as_ref()?;
                let summary = &summaries[&key];
                Some((
                    name.to_ascii_lowercase(),
                    fpas_unit::interface::RoutineInitialization {
                        qualified_name: name.clone(),
                        called_parameters: summary.called_parameters.iter().copied().collect(),
                        started_parameters: summary.started_parameters.iter().copied().collect(),
                        mutable_globals: summary.mutable_globals.iter().cloned().collect(),
                        unknown_worker: summary.unknown_worker
                            || !summary.started_bindings.is_empty(),
                        returned_parameters: summary
                            .returned
                            .iter()
                            .filter_map(|value| match value {
                                walker::Origin::Parameter(index) => Some(*index),
                                _ => None,
                            })
                            .collect(),
                        unknown_call: summary.unknown || !summary.called_bindings.is_empty(),
                        unknown_return: summary
                            .returned
                            .iter()
                            .any(|origin| !matches!(origin, walker::Origin::Parameter(_))),
                    },
                ))
            })
            .collect();
        self.initialization.exported = exported;
        task_failures.sort_by_key(|failure| {
            (
                failure.span.source_id,
                failure.span.offset,
                failure.name.clone(),
            )
        });
        task_failures.dedup_by(|left, right| left.span == right.span && left.name == right.name);
        for failure in task_failures {
            self.error_with_code(fpas_diagnostics::codes::SEMA_FOREIGN_TASK_STATE,
                format!("Worker task cannot access foreign mutable state: {}", failure.name),
                "Pass a value from the starting task, use worker-local variables, or communicate through channels. Make callable dependencies explicit; workers cannot read or write mutable program/unit globals.", failure.span);
        }
        failures.sort_by_key(|failure| {
            (
                failure.span.source_id,
                failure.span.offset,
                failure.name.clone(),
            )
        });
        failures.dedup_by(|left, right| left.span == right.span && left.name == right.name);
        for failure in failures {
            self.error_with_code(fpas_diagnostics::codes::SEMA_UNINITIALIZED_DEPENDENCY,
                format!("Initialization is not safe: {}", failure.name),
                "Initialize the named dependency before this use, or pass already initialized data as an explicit argument. Unknown callable dependencies must be made explicit.", failure.span);
        }
    }
}

//! One typed projection keeps source-call semantics identical across protocol adapters.

use super::model::*;
use crate::{DeclarationOrigin, DocumentSnapshot};
use fpas_diagnostics::SourceSpan;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Protocol-neutral callable data with adapter-specific locations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallableProjection<L> {
    /// Exact declaration identity.
    pub identity: CallableIdentity,
    /// Full declaration or source owner extent.
    pub location: Option<L>,
    /// Tight source selection, when a physical declaration exists.
    pub selection: Option<L>,
    /// Source or native origin.
    pub origin: DeclarationOrigin,
    /// Whether this declaration is within a generic source owner.
    pub generic: bool,
}

/// Written argument position and its checked declaration slot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallArgumentProjection<L> {
    /// Complete argument range.
    pub location: L,
    /// Value excluding a named label.
    pub value: L,
    /// Written parameter label, if any.
    pub name: Option<String>,
    /// Declaration parameter index, counting an implicit receiver.
    pub parameter_index: Option<usize>,
    /// Explicit `var` argument.
    pub by_reference: bool,
}

/// A source call or callable use with exact checked owners.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallRelationProjection<L> {
    /// Source relationship category.
    pub relation: CallRelationKind,
    /// Independent body or declaration-expression owner.
    pub containing: CallableProjection<L>,
    /// Compiler-proven target, when available.
    pub target: Option<CallableProjection<L>>,
    /// Exact written call or value use.
    pub location: L,
    /// Checked consumer context.
    pub context: String,
    /// Ordinary or spawned invocation.
    pub invocation: InvocationKind,
    /// Written argument order and checked parameter mapping.
    pub arguments: Vec<CallArgumentProjection<L>>,
    /// Whether every written argument has a checked parameter slot.
    pub arguments_complete: bool,
    /// Whether the receiver is supplied implicitly.
    pub implicit_receiver: bool,
    /// Callee receiving a callback value, when proven.
    pub receiving: Option<CallableProjection<L>>,
    /// Parameter receiving a callback.
    pub receiving_parameter: Option<usize>,
    /// Missing target or argument evidence.
    pub unresolved_reason: Option<String>,
}

/// Independent relationship collection coverage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallCollectionCoverage {
    /// Number of retained physical uses.
    pub returned: usize,
    /// Exact number after complete semantic traversal.
    pub known_total: Option<usize>,
    /// Whether every eligible matching item is returned.
    pub complete: bool,
}

/// Bounded relationship response shared by CLI and rich editor requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallRelationsProjection<L> {
    /// Proven direct calls.
    pub direct_calls: Vec<CallRelationProjection<L>>,
    /// Callable values outside argument positions.
    pub callable_values: Vec<CallRelationProjection<L>>,
    /// Callback passages, without execution inference.
    pub callback_arguments: Vec<CallRelationProjection<L>>,
    /// Calls without an exact checked target.
    pub dynamic_calls: Vec<CallRelationProjection<L>>,
    /// Completeness of semantic direct-edge analysis, independent of output limits.
    pub direct_edges_complete: bool,
    /// Completeness of exact targets for relevant inventoried invocations.
    pub runtime_targets_complete: bool,
    /// Counts and output coverage for each independent collection.
    pub collections: BTreeMap<String, CallCollectionCoverage>,
}

impl<L> Default for CallRelationsProjection<L> {
    fn default() -> Self {
        Self {
            direct_calls: Vec::new(),
            callable_values: Vec::new(),
            callback_arguments: Vec::new(),
            dynamic_calls: Vec::new(),
            direct_edges_complete: false,
            runtime_targets_complete: false,
            collections: [
                "direct_calls",
                "callable_values",
                "callback_arguments",
                "dynamic_calls",
            ]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    CallCollectionCoverage {
                        returned: 0,
                        known_total: None,
                        complete: false,
                    },
                )
            })
            .collect(),
        }
    }
}
impl<L> CallRelationsProjection<L> {
    /// Shrinks a retained collection for an adapter's byte budget, retaining known totals.
    pub fn trim_for_budget(&mut self) -> Option<&'static str> {
        for (name, items) in [
            ("dynamic_calls", &mut self.dynamic_calls),
            ("callback_arguments", &mut self.callback_arguments),
            ("callable_values", &mut self.callable_values),
            ("direct_calls", &mut self.direct_calls),
        ] {
            if !items.is_empty() {
                items.truncate(items.len() / 2);
                if let Some(coverage) = self.collections.get_mut(name) {
                    coverage.returned = items.len();
                    coverage.complete = false;
                }
                return Some(name);
            }
        }
        None
    }
}

impl CallRelations {
    /// Converts every exact byte span through one adapter-owned location function.
    pub fn project<L, E>(
        &self,
        mut location: impl FnMut(&DocumentSnapshot, SourceSpan) -> Result<L, E>,
    ) -> Result<CallRelationsProjection<L>, E> {
        let mut collections = BTreeMap::new();
        for (name, collection) in [
            ("direct_calls", &self.direct_calls),
            ("callable_values", &self.callable_values),
            ("callback_arguments", &self.callback_arguments),
            ("dynamic_calls", &self.dynamic_calls),
        ] {
            collections.insert(
                name.into(),
                CallCollectionCoverage {
                    returned: collection.items.len(),
                    known_total: collection.known_total,
                    complete: collection.items_complete
                        && collection.known_total == Some(collection.items.len()),
                },
            );
        }
        Ok(CallRelationsProjection {
            direct_calls: project_collection(&self.direct_calls, &mut location)?,
            callable_values: project_collection(&self.callable_values, &mut location)?,
            callback_arguments: project_collection(&self.callback_arguments, &mut location)?,
            dynamic_calls: project_collection(&self.dynamic_calls, &mut location)?,
            direct_edges_complete: self.direct_edges_complete,
            runtime_targets_complete: self.runtime_targets_complete,
            collections,
        })
    }
}
impl SourceCallable {
    /// Projects a physical or native owner without synthesizing a source location.
    pub fn project<L, E>(
        &self,
        location: &mut impl FnMut(&DocumentSnapshot, SourceSpan) -> Result<L, E>,
    ) -> Result<CallableProjection<L>, E> {
        Ok(CallableProjection {
            identity: self.identity.clone(),
            origin: self.origin,
            generic: self.generic,
            location: self
                .snapshot
                .as_ref()
                .zip(self.span)
                .map(|(s, span)| location(s, span))
                .transpose()?,
            selection: self
                .snapshot
                .as_ref()
                .zip(self.selection_span)
                .map(|(s, span)| location(s, span))
                .transpose()?,
        })
    }
}
fn project_collection<L, E>(
    collection: &CallCollection,
    location: &mut impl FnMut(&DocumentSnapshot, SourceSpan) -> Result<L, E>,
) -> Result<Vec<CallRelationProjection<L>>, E> {
    collection
        .items
        .iter()
        .map(|relation| {
            Ok(CallRelationProjection {
                relation: relation.kind,
                containing: relation.containing.project(location)?,
                target: relation
                    .target
                    .as_ref()
                    .map(|t| t.project(location))
                    .transpose()?,
                location: location(&relation.snapshot, relation.span)?,
                context: relation.context.clone(),
                invocation: relation.invocation,
                arguments: relation
                    .arguments
                    .iter()
                    .map(|a| {
                        Ok(CallArgumentProjection {
                            location: location(&relation.snapshot, a.span)?,
                            value: location(&relation.snapshot, a.value_span)?,
                            name: a.name.clone(),
                            parameter_index: a.parameter_index,
                            by_reference: a.by_reference,
                        })
                    })
                    .collect::<Result<_, E>>()?,
                arguments_complete: relation.arguments_complete,
                implicit_receiver: relation.implicit_receiver,
                receiving: relation
                    .receiving
                    .as_ref()
                    .map(|r| r.project(location))
                    .transpose()?,
                receiving_parameter: relation.receiving_parameter,
                unresolved_reason: relation.unresolved_reason.map(str::to_owned),
            })
        })
        .collect()
}

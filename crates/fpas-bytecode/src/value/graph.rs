//! Shared iterative classification and fallible marking of every strong value edge.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`.

use std::collections::HashSet;
use std::sync::Arc;

use super::cell::OwnershipError;
use super::domain::{DomainId, MemoryDomain};
use super::drain::PendingValue;
use super::{ReferenceRoot, Value};
use crate::ExecutableIdentity;
use crate::profile::{self, Counter};

/// Independent ownership, invocation and code properties of a complete graph.
#[derive(Debug, Default)]
pub struct GraphClassification {
    /// Identities of every reachable cell and task-bound callable owner.
    pub owners: HashSet<DomainId>,
    /// A variable reference occurs anywhere in the graph.
    pub has_references: bool,
    /// A callable carries a task-bound invocation restriction.
    pub task_bound: bool,
    /// Code instances required by all contained callables.
    pub executable_identities: HashSet<ExecutableIdentity>,
    /// Shared allocations visited, a measure of the walk's cost.
    pub visited: usize,
}

impl GraphClassification {
    /// Whether the graph contains only values eligible for inter-task transfer.
    ///
    /// Resource handles and target-session code identity need their own admission
    /// checks; this method establishes only local-memory transferability.
    #[must_use]
    pub fn is_transferable(&self) -> bool {
        self.owners.is_empty() && !self.has_references && !self.task_bound
    }

    /// Check that every local owner agrees with the destination domain.
    ///
    /// # Errors
    /// Rejects a graph containing any foreign local cell or callable owner.
    pub fn validate_domain(&self, domain: &MemoryDomain) -> Result<(), OwnershipError> {
        if self.owners.iter().all(|owner| owner == domain.id()) {
            Ok(())
        } else {
            Err(OwnershipError::ForeignDomain)
        }
    }
}

/// Whether construction-time summaries alone prove local-memory transferability.
///
/// A callable that is neither task-bound nor graph-nonlocal captures only neutral
/// values. False is inconclusive; classify the complete graph instead.
#[must_use]
pub fn summarized_transferable(values: &[Value]) -> bool {
    values.iter().all(|value| match value {
        Value::Function(function) => !function.task_bound && function.is_graph_local(),
        value => value.is_ownership_neutral(),
    })
}

/// Classify complete graphs without recursion or deep cloning of wrapper chains.
///
/// Shared aggregates and cells are visited once per allocation. Cell payloads are
/// immutable Arc snapshots held only while their outgoing edges are enumerated.
/// Cached aggregate summaries may skip subgraphs with no ownership or callable edges.
/// Such summaries do not establish resource safety: resource checks must still inspect
/// record layouts even when their fields contain only primitive values.
///
/// # Errors
/// A busy or poisoned cell prevents complete classification and is never guessed.
pub fn classify_values(values: &[Value]) -> Result<GraphClassification, OwnershipError> {
    profile::add(Counter::ClassifyCalls, 1);
    let mut walker = Walker::default();
    walker.visits = Some(Counter::ClassifyVisited);
    for value in values {
        walker.enqueue(value)?;
    }
    walker.finish()
}

/// Classify graphs while observing every contained value, including resource-shaped records.
///
/// Observation disables aggregate-summary shortcuts. Each shared allocation is visited once;
/// wrappers and primitive leaves are borrowed directly without deep cloning. Visit order is
/// unspecified. The observer runs without holding cell locks and may see a partial traversal
/// if a later cell is unavailable. Resource checks can inspect opaque handles and record layouts
/// here without relying on ownership summaries as evidence of resource safety.
///
/// # Errors
/// A busy or poisoned cell prevents complete classification and observation.
pub fn inspect_values(
    values: &[Value],
    mut observer: impl FnMut(&Value),
) -> Result<GraphClassification, OwnershipError> {
    let mut visit = |value: &Value| {
        observer(value);
        Ok(())
    };
    let mut walker = Walker::observing(&mut visit, None);
    for value in values {
        walker.enqueue(value)?;
    }
    walker.finish()
}

/// Validate reachable ownership before changing domain-local storage.
///
/// References remain permitted here for temporary `var` argument registers;
/// retaining them in a mutable cell is rejected separately.
///
/// # Errors
/// Rejects foreign local values or an unavailable graph snapshot.
#[inline]
pub fn validate_local(domain: &MemoryDomain, value: &Value) -> Result<(), OwnershipError> {
    if value.is_ownership_neutral() {
        return Ok(());
    }
    classify_values(std::slice::from_ref(value))?.validate_domain(domain)
}

pub(super) fn validate_storable(
    domain: &MemoryDomain,
    value: &Value,
) -> Result<(), OwnershipError> {
    // Neutral summaries prove the absence of cells, references and callables.
    if value.is_ownership_neutral() {
        return Ok(());
    }
    let classification = classify_values(std::slice::from_ref(value))?;
    classification.validate_domain(domain)?;
    if classification.has_references {
        Err(OwnershipError::EscapingReference)
    } else {
        Ok(())
    }
}

fn insert<T: std::hash::Hash + Eq>(set: &mut HashSet<T>, value: T) -> Result<(), OwnershipError> {
    if !set.contains(&value) {
        set.try_reserve(1)
            .map_err(|_| OwnershipError::ScratchUnavailable)?;
        set.insert(value);
    }
    Ok(())
}

/// Mark cells with the same edge inventory used by validation and inspection.
/// Foreign cells are rejected before taking a snapshot outside collection authority.
/// Returns the marked cells and the number of visited shared allocations.
pub(super) fn reachable_cells(
    domain: &MemoryDomain,
    values: &[Value],
    limit: usize,
) -> Result<(HashSet<usize>, usize), OwnershipError> {
    let mut cells = HashSet::new();
    let visited = trace_collection_roots(domain, values, limit, Some(&mut cells))?;
    Ok((cells, visited))
}

/// Empty candidate sets still require ownership checks, but no cell marking.
/// Returns the number of visited shared allocations.
pub(super) fn validate_collection_roots(
    domain: &MemoryDomain,
    values: &[Value],
    limit: usize,
) -> Result<usize, OwnershipError> {
    trace_collection_roots(domain, values, limit, None)
}

fn trace_collection_roots(
    domain: &MemoryDomain,
    values: &[Value],
    limit: usize,
    mut cells: Option<&mut HashSet<usize>>,
) -> Result<usize, OwnershipError> {
    // Finite diagnostic bounds count every value even when there is nothing to mark.
    let inspect_aggregates = cells.is_some() || limit != usize::MAX;
    let mut visit = |value: &Value| {
        if let Value::Cell(cell) = value {
            if cell.owner() != domain.id() {
                return Err(OwnershipError::ForeignDomain);
            }
            if let Some(cells) = &mut cells {
                insert(cells, cell.identity())?;
            }
        }
        Ok(())
    };
    let mut walker = Walker::observing(&mut visit, Some(limit));
    walker.inspect_aggregates = inspect_aggregates;
    walker.visits = Some(Counter::CollectionVisited);
    for value in values {
        walker.enqueue(value)?;
    }
    let classification = walker.finish()?;
    classification.validate_domain(domain)?;
    Ok(classification.visited)
}

type Observer<'a> = &'a mut dyn FnMut(&Value) -> Result<(), OwnershipError>;

#[derive(Default)]
struct Walker<'a> {
    pending: Vec<Value>,
    visited: HashSet<(u8, usize)>,
    classification: GraphClassification,
    observer: Option<Observer<'a>>,
    remaining: Option<usize>,
    inspect_aggregates: bool,
    /// Profile counter receiving the number of visited allocations.
    visits: Option<Counter>,
}

impl<'a> Walker<'a> {
    fn observing(observer: Observer<'a>, remaining: Option<usize>) -> Self {
        Self {
            pending: Vec::new(),
            visited: HashSet::new(),
            classification: GraphClassification::default(),
            observer: Some(observer),
            remaining,
            inspect_aggregates: true,
            visits: None,
        }
    }

    fn observe(&mut self, value: &Value) -> Result<(), OwnershipError> {
        if let Some(remaining) = &mut self.remaining {
            *remaining = remaining
                .checked_sub(1)
                .ok_or(OwnershipError::ScratchUnavailable)?;
        }
        if let Some(observer) = &mut self.observer {
            observer(value)?;
        }
        Ok(())
    }

    fn enqueue(&mut self, mut value: &Value) -> Result<(), OwnershipError> {
        loop {
            match value {
                Value::Array(values)
                    if !self.inspect_aggregates && values.is_ownership_neutral() =>
                {
                    return Ok(());
                }
                Value::Record(value)
                    if !self.inspect_aggregates && value.is_ownership_neutral() =>
                {
                    return Ok(());
                }
                Value::Enum(value) if !self.inspect_aggregates && value.is_ownership_neutral() => {
                    return Ok(());
                }
                Value::Dict(values)
                    if !self.inspect_aggregates && values.is_ownership_neutral() =>
                {
                    return Ok(());
                }
                Value::ResultOk(payload)
                | Value::ResultError(payload)
                | Value::OptionSome(payload) => {
                    self.observe(value)?;
                    value = payload;
                }
                Value::Array(_)
                | Value::Dict(_)
                | Value::Record(_)
                | Value::Enum(_)
                | Value::Function(_)
                | Value::Cell(_)
                | Value::Reference(_) => {
                    self.pending
                        .try_reserve(1)
                        .map_err(|_| OwnershipError::ScratchUnavailable)?;
                    self.pending.push(value.clone());
                    return Ok(());
                }
                _ => {
                    self.observe(value)?;
                    return Ok(());
                }
            }
        }
    }

    fn finish(mut self) -> Result<GraphClassification, OwnershipError> {
        while let Some(value) = self.pending.pop() {
            let value = PendingValue::new(value);
            let identity = allocation_identity(&value);
            if self.visited.contains(&identity) {
                continue;
            }
            insert(&mut self.visited, identity)?;
            self.observe(&value)?;
            match &*value {
                Value::Array(values) => {
                    for value in values {
                        self.enqueue(value)?;
                    }
                }
                Value::Dict(values) => {
                    for (key, value) in values {
                        self.enqueue(key)?;
                        self.enqueue(value)?;
                    }
                }
                Value::Record(value) => {
                    for value in &value.body().values {
                        self.enqueue(value)?;
                    }
                }
                Value::Enum(value) => {
                    for value in &value.body().values {
                        self.enqueue(value)?;
                    }
                }
                Value::Function(value) => {
                    self.classification.task_bound |= value.task_bound;
                    insert(
                        &mut self.classification.executable_identities,
                        value.target().identity().clone(),
                    )?;
                    if let Some(owner) = &value.owner_domain {
                        insert(&mut self.classification.owners, owner.clone())?;
                    }
                    for capture in &value.captures {
                        self.enqueue(capture)?;
                    }
                    if let Some(receiver) = &value.bound_receiver {
                        self.enqueue(receiver)?;
                    }
                }
                Value::Cell(cell) => {
                    insert(&mut self.classification.owners, cell.owner().clone())?;
                    let snapshot = cell.graph_snapshot()?;
                    self.enqueue(&snapshot)?;
                }
                Value::Reference(value) => {
                    self.classification.has_references = true;
                    match &value.root {
                        ReferenceRoot::Cell(cell) => self.enqueue(&Value::Cell(cell.clone()))?,
                        ReferenceRoot::Global { owner, .. } => {
                            insert(&mut self.classification.owners, owner.clone())?;
                        }
                    }
                }
                _ => unreachable!("only shared graph nodes enter the traversal queue"),
            }
        }
        self.classification.visited = self.visited.len();
        if let Some(counter) = self.visits {
            profile::add(counter, self.visited.len() as u64);
        }
        Ok(std::mem::take(&mut self.classification))
    }
}

impl Drop for Walker<'_> {
    fn drop(&mut self) {
        super::drain::values(&mut self.pending);
    }
}

fn allocation_identity(value: &Value) -> (u8, usize) {
    match value {
        Value::Array(value) => (0, std::ptr::from_ref(&**value) as usize),
        Value::Dict(value) => (1, std::ptr::from_ref(&**value) as usize),
        Value::Record(value) => (2, std::ptr::from_ref(value.body()) as usize),
        Value::Enum(value) => (3, std::ptr::from_ref(value.body()) as usize),
        Value::Function(value) => (4, value.allocation_identity()),
        Value::Cell(value) => (5, value.identity()),
        Value::Reference(value) => (6, Arc::as_ptr(value) as usize),
        _ => unreachable!("only shared graph nodes have allocation identities"),
    }
}

#[cfg(test)]
pub(super) mod tests;

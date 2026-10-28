//! Shared storage for first-class function values.

use super::Value;
use crate::{DomainId, FunctionId, FunctionTarget};
use std::ops::Deref;
use std::sync::Arc;

/// Immutable runtime data for a first-class function value.
///
/// `owner_domain` and executable provenance are runtime-only identities, never serialized into units or
/// programs. **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone)]
pub struct FunctionValue {
    target: FunctionTarget,
    /// Executable target.
    pub function: FunctionId,
    /// Display name; defaults to the executable's shared canonical name.
    ///
    /// Presentation overrides never change the validated target or its code identity.
    pub name: Arc<str>,
    /// Captured values appended to arguments when invoked.
    pub captures: Vec<Value>,
    /// Receiver inserted before visible arguments for a debugger-synthesized bound method.
    pub bound_receiver: Option<Value>,
    /// Whether mutable capture state prevents crossing task boundaries.
    pub task_bound: bool,
    /// Owning runtime domain when `task_bound` is true; otherwise `None`.
    pub owner_domain: Option<DomainId>,
    /// Construction proved that every reachable cell and callable belongs to `owner_domain`
    /// (or that the graph has none), so invocation needs no graph walk.
    ///
    /// Cell payloads change only through writes that validate the same domain.
    local_graph: bool,
}

impl FunctionValue {
    /// Whether construction established an ownership-local capture graph.
    #[must_use]
    pub fn is_graph_local(&self) -> bool {
        self.local_graph
    }

    /// Borrow this callable's validated executable target.
    #[must_use]
    pub fn target(&self) -> &FunctionTarget {
        &self.target
    }
}

impl Drop for FunctionValue {
    fn drop(&mut self) {
        if let Some(receiver) = self.bound_receiver.take() {
            self.captures.push(receiver);
        }
        super::drain::values(&mut self.captures);
    }
}

/// Shared immutable storage for a first-class function value.
#[derive(Debug, Clone)]
pub struct SharedFunction(Arc<FunctionValue>);

impl SharedFunction {
    pub(super) fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut body) = Arc::into_inner(self.0) {
            pending.append(&mut body.captures);
            if let Some(receiver) = body.bound_receiver.take() {
                pending.push(receiver);
            }
        }
    }
    /// Override the presentation name while preserving the validated target and ownership.
    ///
    /// Existing shared copies retain their original name. Invocation always uses [`FunctionValue::target`].
    #[must_use]
    pub fn with_display_name(mut self, name: impl Into<Arc<str>>) -> Self {
        Arc::make_mut(&mut self.0).name = name.into();
        self
    }

    /// Create a non-task-bound function with no runtime task owner.
    pub fn unbound(target: FunctionTarget, captures: Vec<Value>) -> Self {
        Self(Arc::new(FunctionValue {
            function: target.function(),
            name: Arc::clone(target.name()),
            target,
            local_graph: captures.iter().all(Value::is_ownership_neutral),
            captures,
            bound_receiver: None,
            task_bound: false,
            owner_domain: None,
        }))
    }

    /// Create a task-bound function owned by one runtime task.
    pub fn task_owned(
        target: FunctionTarget,
        captures: Vec<Value>,
        owner_domain: DomainId,
    ) -> Self {
        Self(Arc::new(FunctionValue {
            function: target.function(),
            name: Arc::clone(target.name()),
            target,
            local_graph: owned_by(&captures, &owner_domain),
            captures,
            bound_receiver: None,
            task_bound: true,
            owner_domain: Some(owner_domain),
        }))
    }

    /// Create a non-task-bound method value with one receiver argument.
    pub fn bound(target: FunctionTarget, receiver: Value) -> Self {
        Self(Arc::new(FunctionValue {
            function: target.function(),
            name: Arc::clone(target.name()),
            target,
            captures: Vec::new(),
            local_graph: receiver.is_ownership_neutral(),
            bound_receiver: Some(receiver),
            task_bound: false,
            owner_domain: None,
        }))
    }
}

/// Whether every cell and callable reachable from `captures` belongs to `owner`.
fn owned_by(captures: &[Value], owner: &DomainId) -> bool {
    super::graph::classify_values(captures)
        .is_ok_and(|classification| classification.owners.iter().all(|found| found == owner))
}

impl Deref for SharedFunction {
    type Target = FunctionValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

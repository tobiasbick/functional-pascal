//! Shared storage for first-class function values.

use super::Value;
use crate::FunctionId;
use std::ops::Deref;
use std::sync::Arc;

/// Immutable runtime data for a first-class function value.
///
/// `owner_task` is runtime-only identity. It is never serialized into units or
/// programs. **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone)]
pub struct FunctionValue {
    /// Executable target.
    pub function: FunctionId,
    /// Canonical runtime function name.
    pub name: String,
    /// Captured values appended to arguments when invoked.
    pub captures: Vec<Value>,
    /// Whether mutable capture state prevents crossing task boundaries.
    pub task_bound: bool,
    /// Owning runtime task when `task_bound` is true; otherwise `None`.
    pub owner_task: Option<u64>,
}

/// Shared immutable storage for a first-class function value.
#[derive(Debug, Clone)]
pub struct SharedFunction(Arc<FunctionValue>);

impl SharedFunction {
    /// Create a non-task-bound function with no runtime task owner.
    pub fn unbound(function: FunctionId, name: String, captures: Vec<Value>) -> Self {
        Self(Arc::new(FunctionValue {
            function,
            name,
            captures,
            task_bound: false,
            owner_task: None,
        }))
    }

    /// Create a task-bound function owned by one runtime task.
    pub fn task_owned(
        function: FunctionId,
        name: String,
        captures: Vec<Value>,
        owner_task: u64,
    ) -> Self {
        Self(Arc::new(FunctionValue {
            function,
            name,
            captures,
            task_bound: true,
            owner_task: Some(owner_task),
        }))
    }
}

impl Deref for SharedFunction {
    type Target = FunctionValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

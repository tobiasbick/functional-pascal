//! Shared storage for first-class function values.

use super::Value;
use crate::FunctionId;
use std::ops::Deref;
use std::sync::Arc;

/// Immutable runtime data for a first-class function value.
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
}

/// Shared immutable storage for a first-class function value.
#[derive(Debug, Clone)]
pub struct SharedFunction(Arc<FunctionValue>);

impl SharedFunction {
    /// Create a shared function value.
    pub fn new(function: FunctionId, name: String, captures: Vec<Value>, task_bound: bool) -> Self {
        Self(Arc::new(FunctionValue {
            function,
            name,
            captures,
            task_bound,
        }))
    }
}

impl Deref for SharedFunction {
    type Target = FunctionValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

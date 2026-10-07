//! Caller storage named by a `var` argument.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use std::sync::{Arc, Mutex};

use super::Value;

/// Storage that holds the referenced variable.
#[derive(Debug, Clone)]
pub enum ReferenceRoot {
    /// A local variable stored in a shared cell.
    Cell(Arc<Mutex<Value>>),
    /// A mutable global slot.
    Global(u32),
}

/// One step from the root variable to the referenced field or element.
#[derive(Debug, Clone)]
pub enum ReferenceStep {
    /// Positional record field.
    Field(u16),
    /// Array element whose index was evaluated when the reference was created.
    Element(i64),
}

/// A reference to a caller variable, record field, or array element.
///
/// References exist only while the call that received them runs; they never escape
/// into closures, tasks, or stored values.
#[derive(Debug, Clone)]
pub struct VariableReference {
    /// Storage holding the root variable.
    pub root: ReferenceRoot,
    /// Steps from the root to the referenced value.
    pub path: Vec<ReferenceStep>,
}

impl VariableReference {
    /// Returns a reference narrowed by one more step.
    #[must_use]
    pub fn narrowed(&self, step: ReferenceStep) -> Self {
        let mut path = self.path.clone();
        path.push(step);
        Self {
            root: self.root.clone(),
            path,
        }
    }
}

/// Shared handle to a [`VariableReference`].
pub type SharedReference = Arc<VariableReference>;

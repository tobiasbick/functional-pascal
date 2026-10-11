//! Release a finished task scope's retained child results in the scheduler's lock order.
//!
//! Documentation: `docs/pascal/language/concurrency/task-scopes.md`.

use super::TaskScheduler;
use crate::vm::tasks::scopes::ScopeFailure;

impl TaskScheduler {
    /// Close a scope whose children have all finished and discard their unconsumed results.
    pub(in crate::vm) fn poll_scope_close(&self, id: u64) -> Option<Vec<ScopeFailure>> {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let closed = self.scopes.try_close(id)?;
        let mut completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        for task in closed.tasks {
            if results.remove(&task).is_some() {
                completions.insert(task);
            }
        }
        Some(closed.failures)
    }
}

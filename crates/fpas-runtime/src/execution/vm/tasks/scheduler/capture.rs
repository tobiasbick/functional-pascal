//! Bounded identity selection from the scheduler's authoritative retained results.

use super::TaskScheduler;
use crate::execution::vm::TaskResultState;
use std::collections::HashMap;

impl TaskScheduler {
    /// Add the smallest current result identities without copying or consuming their values.
    pub(in crate::execution::vm) fn with_capture_results<T>(
        &self,
        inspect: impl FnOnce(&HashMap<u64, TaskResultState>) -> T,
    ) -> T {
        let results = self.results.lock().unwrap_or_else(|p| p.into_inner());
        inspect(&results)
    }
}

//! Atomic retained-result observations for task waits.
//!
//! Documentation: `docs/pascal/std/concurrency/task.md`.

use super::TaskScheduler;
use crate::vm::{TaskAnyPoll, TaskBatchPoll, TaskResultPoll, TaskResultState};
use fpas_bytecode::Value;

impl TaskScheduler {
    /// Consume one successful retained result, preserving failures and completion identities.
    pub fn poll_result(&self, id: u64) -> TaskResultPoll {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let state = match results.remove(&id) {
            Some(TaskResultState::Pending) => {
                results.insert(id, TaskResultState::Pending);
                return TaskResultPoll::Pending;
            }
            Some(TaskResultState::Failed(error)) => {
                let copy = (*error).clone();
                results.insert(id, TaskResultState::Failed(error));
                return TaskResultPoll::Failed(copy);
            }
            Some(state) => state,
            None if self
                .completions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&id) =>
            {
                return TaskResultPoll::Consumed;
            }
            None => return TaskResultPoll::Unknown,
        };
        self.completions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id);
        TaskResultPoll::Available(match state {
            TaskResultState::Unit => Value::Unit,
            TaskResultState::Value(value) => *value,
            _ => unreachable!(),
        })
    }
    /// Observe every task without consuming successful results.
    pub fn poll_batch(&self, ids: &[u64]) -> TaskBatchPoll {
        let results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(id) = ids
            .iter()
            .find(|id| !results.contains_key(id) && !completions.contains(id))
        {
            return TaskBatchPoll::Unknown(*id);
        }
        if let Some(error) = ids.iter().find_map(|id| match results.get(id) {
            Some(TaskResultState::Failed(e)) => Some((**e).clone()),
            _ => None,
        }) {
            return TaskBatchPoll::Failed(error);
        }
        if ids
            .iter()
            .any(|id| matches!(results.get(id), Some(TaskResultState::Pending)))
        {
            TaskBatchPoll::Pending
        } else {
            TaskBatchPoll::Complete
        }
    }

    /// Observe one completion, validating every identity before selecting in input order.
    pub(in crate::vm) fn poll_any(&self, ids: &[u64]) -> TaskAnyPoll {
        let results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(id) = ids
            .iter()
            .find(|id| !results.contains_key(id) && !completions.contains(id))
        {
            return TaskAnyPoll::Unknown(*id);
        }
        if let Some(error) = ids.iter().find_map(|id| match results.get(id) {
            Some(TaskResultState::Failed(error)) => Some((**error).clone()),
            _ => None,
        }) {
            return TaskAnyPoll::Failed(error);
        }
        match ids
            .iter()
            .position(|id| !matches!(results.get(id), Some(TaskResultState::Pending)))
        {
            Some(index) => TaskAnyPoll::Complete(index),
            None => TaskAnyPoll::Pending,
        }
    }

    /// Sleep until a result changes or queued work becomes available to help.
    pub(in crate::vm) fn wait_for_any(&self, ids: &[u64]) {
        let results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        if ids
            .iter()
            .all(|id| matches!(results.get(id), Some(TaskResultState::Pending)))
            && !self.is_shutdown()
            && self
                .queue
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_empty()
        {
            drop(
                self.results_available
                    .wait(results)
                    .unwrap_or_else(|e| e.into_inner()),
            );
        }
    }
}

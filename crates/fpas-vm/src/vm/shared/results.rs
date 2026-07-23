//! Retained task result registration, polling, and wait coordination.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md`.

use super::SharedState;
use fpas_bytecode::Value;
use std::sync::atomic::Ordering;

pub(crate) enum TaskResultPoll {
    Pending,
    Available(Value),
    Consumed,
    Unknown,
}

pub(crate) enum TaskResultState {
    Pending,
    Unit,
    Value(Box<Value>),
}

impl SharedState {
    /// Register a retained task before it becomes visible to a worker.
    pub(crate) fn register_task_result(&self, id: u64) {
        self.task_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, TaskResultState::Pending);
    }

    /// Store a completed task's return value and notify waiters.
    pub(crate) fn store_task_result(&self, id: u64, value: Value) {
        let state = match value {
            Value::Unit => TaskResultState::Unit,
            value => TaskResultState::Value(Box::new(value)),
        };
        let mut results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        results.insert(id, state);
        self.completed_task_count.fetch_add(1, Ordering::Release);
        drop(results);
        self.task_results_available.notify_all();
    }

    /// Returns `true` when every task id in `task_ids` has a recorded result.
    pub(crate) fn all_tasks_recorded(&self, task_ids: &[u64]) -> bool {
        let results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self
            .task_completions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        task_ids.iter().all(|id| {
            matches!(
                results.get(id),
                Some(TaskResultState::Unit | TaskResultState::Value(_))
            ) || completions.contains(id)
        })
    }

    /// Return the first id that does not belong to a retained task in this VM.
    pub(crate) fn first_unknown_task(&self, task_ids: &[u64]) -> Option<u64> {
        let results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self
            .task_completions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        task_ids
            .iter()
            .copied()
            .find(|id| !results.contains_key(id) && !completions.contains(id))
    }

    /// Consume a completed task result if it is still available.
    pub(crate) fn poll_task_result(&self, id: u64) -> TaskResultPoll {
        let mut task_results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let state = match task_results.entry(id) {
            std::collections::hash_map::Entry::Occupied(entry)
                if matches!(entry.get(), TaskResultState::Pending) =>
            {
                return TaskResultPoll::Pending;
            }
            std::collections::hash_map::Entry::Occupied(entry) => Some(entry.remove()),
            std::collections::hash_map::Entry::Vacant(_) => None,
        };
        let Some(state) = state else {
            if self
                .task_completions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&id)
            {
                return TaskResultPoll::Consumed;
            }
            return TaskResultPoll::Unknown;
        };

        let result = match state {
            TaskResultState::Pending => return TaskResultPoll::Pending,
            TaskResultState::Unit => Value::Unit,
            TaskResultState::Value(value) => *value,
        };
        self.task_completions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id);
        TaskResultPoll::Available(result)
    }

    /// Block until `task_id` has a completion record in [`Self::task_results`] or shutdown.
    ///
    /// Must be paired with [`Self::store_task_result`], which notifies
    /// [`Self::task_results_available`].
    pub(crate) fn wait_until_task_result_ready(&self, task_id: u64) {
        let mut guard = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            match guard.get(&task_id) {
                Some(TaskResultState::Unit | TaskResultState::Value(_)) => return,
                Some(TaskResultState::Pending) => {}
                None if self
                    .task_completions
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .contains(&task_id) =>
                {
                    return;
                }
                None => {}
            }
            if self.is_shutdown() || self.has_ready_task() {
                return;
            }
            guard = self
                .task_results_available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Like [`Self::wait_until_task_result_ready`], but does not wake early when other
    /// tasks are merely queued. Used when the waiter cannot yield (sync callbacks).
    pub(crate) fn wait_until_task_result_ready_strict(&self, task_id: u64) {
        let mut guard = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            match guard.get(&task_id) {
                Some(TaskResultState::Unit | TaskResultState::Value(_)) => return,
                Some(TaskResultState::Pending) => {}
                None if self
                    .task_completions
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .contains(&task_id) =>
                {
                    return;
                }
                None => {}
            }
            if self.is_shutdown() {
                return;
            }
            guard = self
                .task_results_available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Block until every `task_id` has an entry in [`Self::task_results`] (or shutdown).
    pub(crate) fn wait_until_all_tasks_recorded(&self, task_ids: &[u64]) {
        let mut guard = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            let completions = self
                .task_completions
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let missing = task_ids
                .iter()
                .filter(|id| {
                    !matches!(
                        guard.get(id),
                        Some(TaskResultState::Unit | TaskResultState::Value(_))
                    ) && !completions.contains(id)
                })
                .count();
            drop(completions);
            if missing == 0 || self.is_shutdown() {
                return;
            }
            if self.has_ready_task() {
                return;
            }

            let target = self
                .completed_task_count
                .load(Ordering::Acquire)
                .saturating_add(missing as u64);
            while self.completed_task_count.load(Ordering::Acquire) < target && !self.is_shutdown()
            {
                guard = self
                    .task_results_available
                    .wait(guard)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    /// Like [`Self::wait_until_all_tasks_recorded`], but does not wake early for a non-empty
    /// ready queue. Used when the waiter cannot yield (sync callbacks).
    pub(crate) fn wait_until_all_tasks_recorded_strict(&self, task_ids: &[u64]) {
        let mut guard = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            let completions = self
                .task_completions
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let missing = task_ids
                .iter()
                .filter(|id| {
                    !matches!(
                        guard.get(id),
                        Some(TaskResultState::Unit | TaskResultState::Value(_))
                    ) && !completions.contains(id)
                })
                .count();
            drop(completions);
            if missing == 0 || self.is_shutdown() {
                return;
            }

            let target = self
                .completed_task_count
                .load(Ordering::Acquire)
                .saturating_add(missing as u64);
            while self.completed_task_count.load(Ordering::Acquire) < target && !self.is_shutdown()
            {
                guard = self
                    .task_results_available
                    .wait(guard)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    fn has_ready_task(&self) -> bool {
        !self
            .task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }
}

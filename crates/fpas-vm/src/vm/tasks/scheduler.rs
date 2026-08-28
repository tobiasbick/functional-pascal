//! Register-task queue, retained results, timers, and shutdown coordination.

mod completion_ranges;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};

use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use completion_ranges::CompletionRanges;

use crate::vm::{
    TaskBatchPoll, TaskResultPoll, TaskResultState, TaskTimers, VmError, runtime_error,
};

use super::state::TaskState;

/// Outcome of replacing one retained successful result without consuming it.
pub(in crate::vm) enum RetainedResultReplacement {
    /// The available result was replaced atomically.
    Replaced,
    /// The task has not completed yet.
    Pending,
    /// The task currently retains a failure instead of a successful result.
    Failed,
    /// The successful result was already consumed.
    Consumed,
    /// No retained task uses the supplied identity.
    Unknown,
}

/// Mutable scheduler state shared by register workers; executable metadata remains immutable.
pub(in crate::vm) struct TaskScheduler {
    queue: Mutex<VecDeque<TaskState>>,
    available: Condvar,
    timers: TaskTimers<TaskState>,
    accepting_timers: AtomicBool,
    results: Mutex<HashMap<u64, TaskResultState>>,
    completions: Mutex<CompletionRanges>,
    results_available: Condvar,
    next_id: AtomicU64,
    shutdown: AtomicBool,
    abort: AtomicBool,
    first_error: Mutex<Option<VmError>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
            timers: TaskTimers::new(),
            accepting_timers: AtomicBool::new(true),
            results: Mutex::new(HashMap::new()),
            completions: Mutex::new(CompletionRanges::default()),
            results_available: Condvar::new(),
            next_id: AtomicU64::new(1),
            shutdown: AtomicBool::new(false),
            abort: AtomicBool::new(false),
            first_error: Mutex::new(None),
        }
    }

    pub fn alloc_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
    pub fn enqueue(&self, task: TaskState) {
        self.queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(task);
        self.available.notify_one();
        self.results_available.notify_all();
    }
    pub fn try_dequeue(&self) -> Option<TaskState> {
        self.queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop_front()
    }
    pub fn dequeue(&self) -> Option<TaskState> {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
            if self.is_shutdown() {
                return None;
            }
            queue = self
                .available
                .wait(queue)
                .unwrap_or_else(|e| e.into_inner());
        }
    }
    pub fn register_result(&self, id: u64) {
        self.results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, TaskResultState::Pending);
    }
    pub fn store_result(&self, id: u64, value: Value) {
        let state = match value {
            Value::Unit => TaskResultState::Unit,
            value => TaskResultState::Value(Box::new(value)),
        };
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(results.get(&id), Some(TaskResultState::Pending)) {
            results.insert(id, state);
        }
        drop(results);
        self.results_available.notify_all();
    }
    pub fn store_failure(&self, id: u64, error: VmError) {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(results.get(&id), Some(TaskResultState::Pending)) {
            results.insert(id, TaskResultState::Failed(Box::new(error)));
        }
        drop(results);
        self.results_available.notify_all();
    }
    /// Replace one exact retained failure with a pending result.
    pub(in crate::vm) fn recover_failure(&self, id: u64, expected: &VmError) -> bool {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let matches = matches!(
            results.get(&id),
            Some(TaskResultState::Failed(error)) if error.as_ref() == expected
        );
        if matches {
            results.insert(id, TaskResultState::Pending);
        }
        drop(results);
        if matches {
            self.results_available.notify_all();
        }
        matches
    }
    /// Replace one exact retained failure with a successful result.
    pub(in crate::vm) fn replace_failure(&self, id: u64, expected: &VmError, value: Value) -> bool {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let matches = matches!(
            results.get(&id),
            Some(TaskResultState::Failed(error)) if error.as_ref() == expected
        );
        if matches {
            let replacement = match value {
                Value::Unit => TaskResultState::Unit,
                value => TaskResultState::Value(Box::new(value)),
            };
            results.insert(id, replacement);
        }
        drop(results);
        if matches {
            self.results_available.notify_all();
        }
        matches
    }
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
    /// Replace one available successful result without consuming it.
    pub(in crate::vm) fn replace_available_result(
        &self,
        id: u64,
        value: Value,
    ) -> RetainedResultReplacement {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        let outcome = match results.get(&id) {
            Some(TaskResultState::Unit | TaskResultState::Value(_)) => {
                let replacement = match value {
                    Value::Unit => TaskResultState::Unit,
                    value => TaskResultState::Value(Box::new(value)),
                };
                results.insert(id, replacement);
                RetainedResultReplacement::Replaced
            }
            Some(TaskResultState::Pending) => RetainedResultReplacement::Pending,
            Some(TaskResultState::Failed(_)) => RetainedResultReplacement::Failed,
            None => {
                let consumed = self
                    .completions
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .contains(&id);
                if consumed {
                    RetainedResultReplacement::Consumed
                } else {
                    RetainedResultReplacement::Unknown
                }
            }
        };
        drop(results);
        if matches!(outcome, RetainedResultReplacement::Replaced) {
            self.results_available.notify_all();
        }
        outcome
    }
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
    #[cfg(test)]
    fn consumed_completion_storage_len(&self) -> usize {
        self.completions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .range_count()
    }
    pub fn wait_for_result(&self, id: u64) {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        while matches!(results.get(&id), Some(TaskResultState::Pending)) && !self.is_shutdown() {
            results = self
                .results_available
                .wait(results)
                .unwrap_or_else(|e| e.into_inner());
        }
    }
    pub fn wait_for_batch(&self, ids: &[u64]) {
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        while ids
            .iter()
            .any(|id| matches!(results.get(id), Some(TaskResultState::Pending)))
            && !self.is_shutdown()
        {
            results = self
                .results_available
                .wait(results)
                .unwrap_or_else(|e| e.into_inner());
        }
    }
    /// Complete one still-pending retained result after scheduler shutdown.
    pub fn fail_pending_result_if_shutdown(&self, task_id: u64) {
        if !self.is_shutdown() {
            return;
        }
        let error = self
            .first_error()
            .unwrap_or_else(|| self.shutdown_error(task_id));
        self.store_failure(task_id, error);
    }
    /// Complete all still-pending retained results in a wait batch after shutdown.
    pub fn fail_pending_batch_if_shutdown(&self, task_ids: &[u64]) {
        for task_id in task_ids {
            self.fail_pending_result_if_shutdown(*task_id);
        }
    }
    pub fn schedule(&self, task: TaskState, milliseconds: u64) {
        if let Err(task) = self
            .timers
            .schedule(task, milliseconds, &self.accepting_timers)
        {
            self.cancel(task);
        }
    }
    pub fn timer_loop(&self) {
        while self.timers.dispatch_next_due(&self.shutdown, |tasks| {
            for task in tasks {
                self.enqueue(task);
            }
        }) {}
    }
    pub fn finish_main(&self) {
        self.accepting_timers.store(false, Ordering::Release);
        for task in self.timers.cancel_all() {
            self.cancel(task);
        }
        self.shutdown.store(true, Ordering::Release);
        self.complete_pending_results();
        self.timers.notify_shutdown();
        self.available.notify_all();
        self.results_available.notify_all();
    }
    /// Complete one retained task with the standard runtime-shutdown diagnostic.
    pub(in crate::vm) fn cancel_result(&self, task_id: u64) {
        self.store_failure(task_id, self.shutdown_error(task_id));
    }
    pub fn fail(&self, error: VmError) {
        *self.first_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(error);
        self.abort.store(true, Ordering::Release);
        self.finish_main();
    }
    pub fn request_cancel(&self) {
        self.abort.store(true, Ordering::Release);
        self.finish_main();
    }
    pub fn first_error(&self) -> Option<VmError> {
        self.first_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
    pub fn is_aborted(&self) -> bool {
        self.abort.load(Ordering::Acquire)
    }
    fn cancel(&self, task: TaskState) {
        if task.retain_result {
            self.cancel_result(task.id);
        }
    }
    fn shutdown_error(&self, task_id: u64) -> VmError {
        runtime_error(
            RUNTIME_VM_SHUTDOWN,
            format!("Task {task_id} was canceled because the runtime shut down"),
            "Wait for retained tasks before the main task finishes.",
            SourceLocation::new(1, 1),
        )
    }
    fn complete_pending_results(&self) {
        let first_error = self.first_error();
        let mut results = self.results.lock().unwrap_or_else(|e| e.into_inner());
        for (task_id, state) in results.iter_mut() {
            if matches!(state, TaskResultState::Pending) {
                let error = first_error
                    .clone()
                    .unwrap_or_else(|| self.shutdown_error(*task_id));
                *state = TaskResultState::Failed(Box::new(error));
            }
        }
    }
}

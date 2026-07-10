//! Shared state accessible by all worker threads.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 3), `docs/pascal/language/concurrency/README.md`.
//!
//! ## Lock ordering
//!
//! Independent mutexes (`task_queue`, `task_results`, `console`, `text_input`, `key_input`, `tui`, `graph`)
//! each protect a single concern. **Do not acquire more than one of these locks at the same time**
//! from VM or intrinsic code unless the order is documented here and consistently followed.
//! [`RwLock`] on `globals` is separate; avoid holding `globals` while waiting on `task_available`.
//! Pool workers follow [`super::Worker::pool_loop`]: take `task_queue`, then wait on
//! `task_available` while holding that guard (standard `Condvar` pattern).
//! `TaskWait` / `WaitAll` must not use that same wait for **result** readiness: notifications from
//! [`Self::store_task_result`] are paired with [`Self::task_results_available`] while holding
//! [`Self::task_results`] so wakeups cannot be missed between a poll and a block.

use fpas_bytecode::{Chunk, Value};
use fpas_std::{Console, KeyInput, TextInput};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, RwLock};
#[cfg(test)]
use std::time::Duration;

mod graph;
mod tui;

pub(crate) use graph::GraphState;
pub(crate) use tui::{
    TuiState, TurboVisionMenu, TurboVisionMenuItem, TurboVisionOutlineNode, TurboVisionRect,
    TurboVisionStatusItem,
};

pub(crate) enum TaskResultPoll {
    Pending,
    Available(Value),
    Consumed,
}

pub(crate) enum TaskResultState {
    Available(Value),
}

/// Shared state for the parallel VM.
///
/// All fields are thread-safe. Workers hold `Arc<SharedState>` and
/// access individual fields through the appropriate synchronization primitive.
pub(crate) struct SharedState {
    /// Compiled bytecode (read-only after construction).
    pub chunk: Chunk,

    /// Process arguments visible to `Std.Args` (read-only after construction).
    pub program_args: Vec<String>,

    /// Global variables.
    pub globals: RwLock<HashMap<String, Value>>,

    /// Ready queue of suspended tasks (FIFO).
    pub task_queue: Mutex<VecDeque<TaskState>>,
    /// Signalled when new tasks are pushed or existing tasks become ready.
    pub task_available: Condvar,

    /// Completed task states for tasks whose results can still be observed.
    pub task_results: Mutex<HashMap<u64, TaskResultState>>,
    /// Task ids whose results were consumed by `Wait` but may still be observed by `WaitAll`.
    pub task_completions: Mutex<HashSet<u64>>,
    /// Woken when [`Self::task_results`] gains or updates an entry (for example after
    /// [`Self::store_task_result`]). Always wait while holding the [`Self::task_results`] mutex.
    pub task_results_available: Condvar,
    /// Next task id (monotonically increasing; 0 = main program).
    pub next_task_id: AtomicU64,

    /// Console output (shared, mutex-protected).
    pub console: Mutex<Console>,
    /// Line-buffered stdin.
    pub text_input: Mutex<TextInput>,
    /// CRT-style keyboard buffer.
    pub key_input: Mutex<KeyInput>,
    /// Minimal shared `Std.Tui` application/session state.
    pub tui: Mutex<TuiState>,
    /// Minimal shared `Std.Graph` application/session state.
    pub graph: Mutex<GraphState>,

    /// Set when the main task completes or an error occurs.
    pub shutdown: AtomicBool,

    /// Set when any worker hits a runtime error so in-flight spawned tasks cooperatively exit.
    /// Plain [`Self::request_shutdown`] (after the main task finishes) does **not** set this flag,
    /// so workers can still run tasks that were queued before teardown.
    pub abort_spawned_bytecode: AtomicBool,
}

/// Saved state of a suspended task (ready to be resumed by any worker).
pub(crate) struct TaskState {
    pub id: u64,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub call_stack: Vec<super::CallFrame>,
    pub retain_result: bool,
}

impl SharedState {
    /// Allocate a fresh task id.
    ///
    /// IDs are monotonically increasing `u64` values starting at 1 (0 is reserved for the main
    /// task).  After ~18 quintillion allocations the counter wraps to 0.  In practice this is
    /// unreachable, but callers should not store IDs across a restart.
    pub(crate) fn alloc_task_id(&self) -> u64 {
        self.next_task_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Push a task onto the ready queue and notify one waiting worker.
    pub(crate) fn enqueue_task(&self, task: TaskState) {
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_back(task);
        self.task_available.notify_one();
    }

    /// Pop the oldest ready task from the queue (returns `None` if empty).
    pub(crate) fn try_dequeue_task(&self) -> Option<TaskState> {
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop_front()
    }

    /// Store a completed task's return value and notify waiters.
    pub(crate) fn store_task_result(&self, id: u64, value: Value) {
        self.task_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, TaskResultState::Available(value));
        self.task_results_available.notify_all();
        // Legacy: tests and pool code may wait on `task_available` while holding `task_queue`.
        self.task_available.notify_all();
    }

    /// Returns `true` when every task id in `task_ids` has a recorded result.
    pub(crate) fn all_tasks_recorded(&self, task_ids: &[u64]) -> bool {
        let results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self
            .task_completions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        task_ids
            .iter()
            .all(|id| results.contains_key(id) || completions.contains(id))
    }

    /// Consume a completed task result if it is still available.
    pub(crate) fn poll_task_result(&self, id: u64) -> TaskResultPoll {
        let mut task_results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = task_results.get_mut(&id) else {
            if self
                .task_completions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&id)
            {
                return TaskResultPoll::Consumed;
            }
            return TaskResultPoll::Pending;
        };

        match state {
            TaskResultState::Available(value) => {
                let result = value.clone();
                task_results.remove(&id);
                self.task_completions
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(id);
                TaskResultPoll::Available(result)
            }
        }
    }

    /// Signal all workers to shut down.
    pub(crate) fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.task_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.task_completions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.task_available.notify_all();
        self.task_results_available.notify_all();
    }

    /// Signal global shutdown **and** request in-flight spawned tasks to stop before the next
    /// instruction boundary (after a concurrent task failure).
    pub(crate) fn signal_runtime_failure(&self) {
        self.abort_spawned_bytecode.store(true, Ordering::Release);
        self.request_shutdown();
    }

    /// Block until notified (task queued, result stored, or shutdown).
    ///
    /// `timeout`: `None` waits until [`Condvar::wait`] returns; `Some(d)` uses [`Condvar::wait_timeout`].
    /// A zero duration returns immediately without blocking.
    ///
    /// Used for **ready-queue** progress (pool workers, enqueue tests). `TaskWait` uses
    /// [`Self::wait_until_task_result_ready`] / [`Self::wait_until_all_tasks_recorded`] instead so
    /// result notifications cannot be lost between a poll and a sleep.
    #[cfg(test)]
    pub(crate) fn wait_for_task_progress(&self, timeout: Option<Duration>) {
        let queue = self.task_queue.lock().unwrap_or_else(|e| e.into_inner());
        match timeout {
            None => {
                let _guard = self
                    .task_available
                    .wait(queue)
                    .unwrap_or_else(|e| e.into_inner());
            }
            Some(d) if d.is_zero() => {}
            Some(d) => {
                let _guard = self
                    .task_available
                    .wait_timeout(queue, d)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
    }

    /// Block until `task_id` has a completion record in [`Self::task_results`] or shutdown.
    ///
    /// Must be paired with [`Self::store_task_result`], which notifies [`Self::task_results_available`].
    pub(crate) fn wait_until_task_result_ready(&self, task_id: u64) {
        let mut guard = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            match guard.get(&task_id) {
                Some(TaskResultState::Available(_)) => return,
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
            let all = task_ids
                .iter()
                .all(|id| guard.contains_key(id) || completions.contains(id));
            drop(completions);
            if all || self.is_shutdown() {
                return;
            }
            guard = self
                .task_results_available
                .wait(guard)
                .unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Check whether shutdown has been requested.
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
}

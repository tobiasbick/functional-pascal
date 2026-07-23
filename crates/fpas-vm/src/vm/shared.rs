//! Shared state accessible by all worker threads.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 3), `docs/pascal/language/concurrency/README.md`.
//!
//! ## Lock ordering
//!
//! Independent mutexes (`task_queue`, `task_results`, `console`, `text_input`, `key_input`, `graph`)
//! each protect a single concern. **Do not acquire more than one of these locks at the same time**
//! from VM or intrinsic code unless the order is documented here and consistently followed.
//! [`RwLock`] on `globals` is separate; avoid holding `globals` while waiting on `task_available`.
//! Result waits may take `task_results` and then briefly inspect `task_queue`; no path may acquire
//! those two locks in the reverse order. Timer wakeup drops `task_queue` before taking
//! `task_results` to notify a result waiter that runnable work exists.
//! Pool workers follow [`super::Worker::pool_loop`]: take `task_queue`, then wait on
//! `task_available` while holding that guard (standard `Condvar` pattern).
//! `TaskWait` / `WaitAll` must not use that same wait for **result** readiness: notifications from
//! [`Self::store_task_result`] are paired with [`Self::task_results_available`] while holding
//! [`Self::task_results`] so wakeups cannot be missed between a poll and a block.

use fpas_bytecode::{Chunk, Value};
use fpas_std::{Console, KeyInput, TextInput};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
#[cfg(test)]
use std::time::Duration;

mod graph;
mod results;
mod timers;

pub(crate) use results::{TaskResultPoll, TaskResultState};
pub(crate) use timers::TaskTimers;

pub(crate) use graph::GraphState;

/// Shared state for the parallel VM.
///
/// All fields are thread-safe. Workers hold `Arc<SharedState>` and
/// access individual fields through the appropriate synchronization primitive.
pub(crate) struct SharedState {
    /// Compiled bytecode (read-only after construction).
    pub chunk: Arc<Chunk>,

    /// Process arguments visible to `Std.Args` (read-only after construction).
    pub program_args: Vec<String>,

    /// Global variables.
    pub globals: RwLock<HashMap<String, Value>>,

    /// Ready queue of suspended tasks (FIFO).
    pub task_queue: Mutex<VecDeque<TaskState>>,
    /// Signalled when new tasks are pushed or existing tasks become ready.
    pub task_available: Condvar,

    /// Spawned tasks suspended by cooperative `Std.Time.Sleep`.
    pub task_timers: TaskTimers,

    /// Completed task states for tasks whose results can still be observed.
    pub task_results: Mutex<HashMap<u64, TaskResultState>>,
    /// Task ids whose results were consumed by `Wait` but may still be observed by `WaitAll`.
    pub task_completions: Mutex<HashSet<u64>>,
    /// Woken when [`Self::task_results`] gains or updates an entry (for example after
    /// [`Self::store_task_result`]). Always wait while holding the [`Self::task_results`] mutex.
    pub task_results_available: Condvar,
    /// Number of retained tasks that have recorded a result during this VM lifetime.
    pub completed_task_count: AtomicU64,
    /// Next task id (monotonically increasing; 0 = main program).
    pub next_task_id: AtomicU64,

    /// Console output (shared, mutex-protected).
    pub console: Mutex<Console>,
    /// Line-buffered stdin.
    pub text_input: Mutex<TextInput>,
    /// CRT-style keyboard buffer.
    pub key_input: Mutex<KeyInput>,
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

    /// Append a due timer bucket to the ready queue and wake pool workers.
    pub(crate) fn enqueue_tasks(&self, tasks: Vec<TaskState>) {
        if tasks.is_empty() {
            return;
        }
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(tasks);
        self.task_available.notify_all();
        // A spawned task may be waiting for a sleeping child while occupying the only pool
        // worker. Pair this notification with the result mutex so it cannot be lost between the
        // waiter's ready-queue check and its condvar wait.
        let _results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        self.task_results_available.notify_all();
    }

    /// Suspend a spawned task without holding its pool worker.
    pub(crate) fn schedule_task_after(&self, task: TaskState, milliseconds: u64) {
        self.task_timers.schedule(task, milliseconds);
    }

    /// Move due timer buckets to the shared ready queue until shutdown.
    pub(crate) fn timer_loop(&self) {
        while let Some(tasks) = self.task_timers.wait_for_due(&self.shutdown) {
            self.enqueue_tasks(tasks);
        }
    }

    /// Pop the oldest ready task from the queue (returns `None` if empty).
    pub(crate) fn try_dequeue_task(&self) -> Option<TaskState> {
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop_front()
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
        self.task_timers.notify_shutdown();
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

    /// Check whether shutdown has been requested.
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
}

//! Shared state accessible by all worker threads.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 3), `docs/pascal/08-concurrency.md`.
//!
//! ## Lock ordering
//!
//! Independent mutexes (`task_queue`, `task_results`, `console`, `text_input`, `key_input`, `tui`)
//! each protect a single concern. **Do not acquire more than one of these locks at the same time**
//! from VM or intrinsic code unless the order is documented here and consistently followed.
//! [`RwLock`] on `globals` is separate; avoid holding `globals` while waiting on `task_available`.
//! Pool workers follow [`super::Worker::pool_loop`]: take `task_queue`, then wait on
//! `task_available` while holding that guard (standard `Condvar` pattern).
//! `TaskWait` / `WaitAll` must not use that same wait for **result** readiness: notifications from
//! [`Self::store_task_result`] are paired with [`Self::task_results_available`] while holding
//! [`Self::task_results`] so wakeups cannot be missed between a poll and a block.

use fpas_bytecode::{Chunk, Value};
use fpas_std::{Console, KeyInput, TextInput, TuiHost, TuiSession};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, RwLock};
#[cfg(test)]
use std::time::Duration;

pub(crate) enum TaskResultPoll {
    Pending,
    Available(Value),
    Consumed,
}

pub(crate) enum TaskResultState {
    Available(Value),
    Consumed,
}

#[derive(Debug)]
pub(crate) struct TuiState {
    pub session: TuiSession,
    /// Resize coalescing and future hosted-loop pump (`docs/pascal/std/tui-app.md`).
    pub host: TuiHost,
    /// `OnKeyPressed`-style handler: `function (Application, KeyEvent): boolean`.
    pub on_key_pressed: Option<Value>,
    /// `OnResize`-style handler: `procedure (Application, Size)` (two arguments).
    pub on_resize: Option<Value>,
    /// `OnPaint`-style handler: `procedure (Application)` (one argument).
    pub on_paint: Option<Value>,
    /// `OnIdle`-style handler: `procedure (Application)` (one argument).
    pub on_idle: Option<Value>,
    /// Idle interval for hosted `Application.Run` callbacks in milliseconds; `0` disables idle.
    pub idle_interval_ms: i64,
    /// `OnExit`-style handler: `procedure (Application, ExitReason)` (registered when `Application.Run` / bridge exists).
    pub on_exit: Option<Value>,
    /// Last reason recorded for a hosted run (`Std.Tui.ExitReason` enum value); set when a future `Run` loop stops.
    pub last_exit_reason: Option<Value>,
    /// Set by `TuiHostRequestQuit`; consumed when [`crate::vm::execute::io::tui::Worker`] run loop observes it.
    pub quit_requested: bool,
    /// Set when low-level code asks the active hosted `Application.Run` session to stop.
    pub host_stop_requested: bool,
    /// Guards the single hosted `Application.Run` entrypoint for the active session.
    pub run_active: bool,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            session: TuiSession::default(),
            host: TuiHost::new(),
            on_key_pressed: None,
            on_resize: None,
            on_paint: None,
            on_idle: None,
            idle_interval_ms: 0,
            on_exit: None,
            last_exit_reason: None,
            quit_requested: false,
            host_stop_requested: false,
            run_active: false,
        }
    }
}

/// Shared state for the parallel VM.
///
/// All fields are thread-safe. Workers hold `Arc<SharedState>` and
/// access individual fields through the appropriate synchronization primitive.
pub(crate) struct SharedState {
    /// Compiled bytecode (read-only after construction).
    pub chunk: Chunk,

    /// Global variables.
    pub globals: RwLock<HashMap<String, Value>>,

    /// Ready queue of suspended tasks.
    pub task_queue: Mutex<Vec<TaskState>>,
    /// Signalled when new tasks are pushed or existing tasks become ready.
    pub task_available: Condvar,

    /// Completed task states for tasks whose results can still be observed.
    pub task_results: Mutex<HashMap<u64, TaskResultState>>,
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
    pub(crate) fn alloc_task_id(&self) -> u64 {
        self.next_task_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Push a task onto the ready queue and notify one waiting worker.
    pub(crate) fn enqueue_task(&self, task: TaskState) {
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(task);
        self.task_available.notify_one();
    }

    /// Pop a ready task from the queue (returns `None` if empty).
    pub(crate) fn try_dequeue_task(&self) -> Option<TaskState> {
        self.task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .pop()
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

    /// Returns true once a retained task has finished and an entry exists in [`Self::task_results`].
    ///
    /// The entry remains after [`Self::poll_task_result`] consumes the value (state becomes
    /// [`TaskResultState::Consumed`]); callers that need a still-available result must use
    /// [`Self::poll_task_result`].
    pub(crate) fn task_completion_recorded(&self, id: u64) -> bool {
        self.task_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(&id)
    }

    /// Consume a completed task result if it is still available.
    pub(crate) fn poll_task_result(&self, id: u64) -> TaskResultPoll {
        let mut task_results = self.task_results.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = task_results.get_mut(&id) else {
            return TaskResultPoll::Pending;
        };

        match state {
            TaskResultState::Available(value) => {
                let result = value.clone();
                *state = TaskResultState::Consumed;
                TaskResultPoll::Available(result)
            }
            TaskResultState::Consumed => TaskResultPoll::Consumed,
        }
    }

    /// Signal all workers to shut down.
    pub(crate) fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
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
                Some(TaskResultState::Available(_)) | Some(TaskResultState::Consumed) => return,
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
            let all = task_ids.iter().all(|id| guard.contains_key(id));
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

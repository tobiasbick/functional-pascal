//! Shared state accessible by all worker threads.
//!
//! **Documentation:** `docs/future/parallel-vm.md`

use crossbeam_channel as cbc;
use fpas_bytecode::{Chunk, Value};
use fpas_std::{Console, KeyInput, TextInput};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, RwLock};

/// Per-channel state shared across workers.
pub(crate) struct SharedChannel {
    pub sender: cbc::Sender<Value>,
    pub receiver: cbc::Receiver<Value>,
    pub closed: AtomicBool,
}

pub(crate) enum TaskResultPoll {
    Pending,
    Available(Value),
    Consumed,
}

pub(crate) enum TaskResultState {
    Available(Value),
    Consumed,
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

    /// Channel registry: channel id → shared channel.
    pub channels: Mutex<HashMap<u64, SharedChannel>>,
    /// Next channel id (monotonically increasing).
    pub next_channel_id: AtomicU64,

    /// Ready queue of suspended tasks.
    pub task_queue: Mutex<Vec<TaskState>>,
    /// Signalled when new tasks are pushed or existing tasks become ready.
    pub task_available: Condvar,

    /// Completed task states for tasks whose results can still be observed.
    pub task_results: Mutex<HashMap<u64, TaskResultState>>,
    /// Next task id (monotonically increasing; 0 = main program).
    pub next_task_id: AtomicU64,

    /// Console output (shared, mutex-protected).
    pub console: Mutex<Console>,
    /// Line-buffered stdin.
    pub text_input: Mutex<TextInput>,
    /// CRT-style keyboard buffer.
    pub key_input: Mutex<KeyInput>,

    /// Set when the main task completes or an error occurs.
    pub shutdown: AtomicBool,
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

    /// Allocate a fresh channel id.
    pub(crate) fn alloc_channel_id(&self) -> u64 {
        self.next_channel_id.fetch_add(1, Ordering::Relaxed)
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
    }

    /// Check whether a task has already completed.
    pub(crate) fn has_task_result(&self, id: u64) -> bool {
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
    }

    /// Check whether shutdown has been requested.
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
}

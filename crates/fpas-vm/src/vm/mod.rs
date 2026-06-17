//! Parallel stack VM for FPAS bytecode.
//!
//! The VM executes the main program on the calling thread. Tasks created with
//! `go` are distributed across a thread pool for true parallel execution.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`, `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, SourceLocation};
use fpas_std::{
    Console, ConsoleEvent, ConsoleKeyEvent, GraphEvent, KeyInput, ScreenSnapshot, TextInput,
};
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Condvar, Mutex, RwLock};

mod diagnostics;
mod execute;
mod helpers;
mod shared;
mod shutdown;
mod worker;

pub use diagnostics::VmError;
pub(crate) use diagnostics::{internal_error, runtime_error};
pub(crate) use shared::{GraphState, SharedState, TaskResultPoll, TaskState, TuiState};
pub use shutdown::VmShutdownHandle;
pub(crate) use worker::Worker;

const STACK_MAX: usize = 4096;
const TIMESLICE: u32 = 256;

/// Drops after [`Worker::run`] returns or unwinds so [`SharedState::request_shutdown`] always runs.
/// Pool workers block on the task condvar; without this, a panic in the main worker could strand them.
///
/// **Documentation:** `docs/pascal/08-concurrency.md`.
struct ShutdownAfterMain(Arc<SharedState>);

impl Drop for ShutdownAfterMain {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

pub(crate) fn canonical_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Re-export captured output type from fpas-std.
pub type VmOutput = fpas_std::CapturedOutput;

/// Call frame for function invocations.
#[derive(Debug)]
pub(crate) struct CallFrame {
    /// Return address (instruction pointer to resume after call).
    pub return_ip: usize,
    /// Base slot of this frame on the value stack.
    pub base_slot: usize,
}

/// Public VM interface.
///
/// Holds shared state and provides the entry point for program execution.
/// Internally uses `Worker` threads for parallel task execution.
pub struct Vm {
    shared: Arc<SharedState>,
    /// Pool size for worker threads (0 = main-thread only until first `go`).
    pool_size: usize,
}

impl Vm {
    /// Create a new VM (output is captured, not streamed).
    pub fn new(chunk: Chunk) -> Self {
        Self::build(chunk, Console::new(), Vec::new())
    }

    /// Create a new VM with process arguments visible through `Std.Args`.
    pub fn with_args(chunk: Chunk, args: Vec<String>) -> Self {
        Self::build(chunk, Console::new(), args)
    }

    /// Create a VM that streams output to the given writer immediately.
    pub fn with_writer(chunk: Chunk, writer: Box<dyn Write + Send>) -> Self {
        Self::build(chunk, Console::with_writer(writer), Vec::new())
    }

    /// Create a VM that streams output and exposes process arguments through `Std.Args`.
    pub fn with_writer_and_args(
        chunk: Chunk,
        writer: Box<dyn Write + Send>,
        args: Vec<String>,
    ) -> Self {
        Self::build(chunk, Console::with_writer(writer), args)
    }

    fn build(chunk: Chunk, console: Console, program_args: Vec<String>) -> Self {
        // **Documentation:** `docs/pascal/08-concurrency.md` (from the repository root).
        //
        // Spawning `available_parallelism() - 1` workers for every `Vm::run()` caused severe
        // slowdown when hundreds of tests each created a VM in parallel: each run blocked one
        // idle worker thread on a condvar. Only programs that emit `SpawnTask` / `SpawnDetachedTask`
        // need a pool.
        let pool_size = if chunk.uses_spawn_tasks() {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .saturating_sub(1)
                .max(1)
        } else {
            0
        };

        let shared = Arc::new(SharedState {
            chunk,
            program_args,
            globals: RwLock::new(HashMap::new()),
            task_queue: Mutex::new(Vec::new()),
            task_available: Condvar::new(),
            task_results: Mutex::new(HashMap::new()),
            task_results_available: Condvar::new(),
            next_task_id: AtomicU64::new(1),
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            tui: Mutex::new(TuiState::default()),
            graph: Mutex::new(GraphState::default()),
            shutdown: AtomicBool::new(false),
            abort_spawned_bytecode: AtomicBool::new(false),
        });

        Self { shared, pool_size }
    }

    /// Test-only accessor for [`Self::build`] worker count (spawn chunks only).
    #[cfg(test)]
    pub(crate) fn worker_pool_size_for_tests(&self) -> usize {
        self.pool_size
    }

    /// Test-only: whether global shutdown was requested after a run (or mid-run failure).
    #[cfg(test)]
    pub(crate) fn is_shutdown_for_tests(&self) -> bool {
        self.shared.is_shutdown()
    }

    /// Queue a line for the next line-buffered `Read` / `ReadLn` (tests).
    pub fn push_readln_input(&mut self, line: &str) {
        self.shared
            .text_input
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_line(line);
    }

    /// Queue characters for the next `Std.Console.ReadKey` calls (tests).
    pub fn push_readkey_input(&mut self, s: &str) {
        self.shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_chars(s);
    }

    /// Queue a structured key for the next `Std.Console.ReadKeyEvent` (tests).
    pub fn push_key_event(&mut self, ev: ConsoleKeyEvent) {
        self.shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_key_event(ev);
    }

    /// Queue a structured console event for the next `Std.Console.ReadEvent` (tests).
    pub fn push_console_event(&mut self, ev: ConsoleEvent) {
        self.shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push_console_event(ev);
    }

    /// Queue a structured graph event for the next `Std.Graph.Application.PollEvent` (tests).
    pub fn push_graph_event(&mut self, ev: GraphEvent) {
        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        if graph
            .session
            .push_event(ev.clone(), SourceLocation::new(1, 1))
            .is_err()
        {
            graph.pending_test_events.push(ev);
        }
    }

    /// Access captured output (for test assertions).
    pub fn output(&self) -> VmOutput {
        self.shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .clone()
    }

    /// Returns the current logical CRT screen snapshot (for test assertions).
    pub fn screen_snapshot(&self) -> ScreenSnapshot {
        self.shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .screen_snapshot()
    }

    /// Returns a handle for cooperative shutdown while [`Self::run`] executes on another thread.
    pub fn shutdown_handle(&self) -> VmShutdownHandle {
        VmShutdownHandle::new(Arc::clone(&self.shared))
    }

    /// Execute the loaded program.
    ///
    /// The main program runs on the calling thread. If `go` tasks are spawned,
    /// a thread pool is created to execute them in parallel.
    pub fn run(&mut self) -> Result<(), VmError> {
        self.shared
            .abort_spawned_bytecode
            .store(false, std::sync::atomic::Ordering::Release);

        // Programs without `go` never use the pool; avoid `thread::scope` so nested callers
        // (for example `fpas test --jobs`) do not stack scoped thread blocks on worker threads.
        if self.pool_size == 0 {
            let _shutdown_after_main = ShutdownAfterMain(Arc::clone(&self.shared));
            let mut main_worker = Worker::new_main(Arc::clone(&self.shared));
            return main_worker.run();
        }

        let shared = Arc::clone(&self.shared);
        let pool_size = self.pool_size;

        // Spawn pool workers in a scoped thread block.
        std::thread::scope(|scope| {
            // Spawn pool worker threads.
            let mut handles = Vec::with_capacity(pool_size);
            for _ in 0..pool_size {
                let s = Arc::clone(&shared);
                handles.push(scope.spawn(move || {
                    let mut w = Worker::new_pool(s);
                    w.pool_loop()
                }));
            }

            // Run main program on this thread. Always shut down when the main worker returns or
            // unwinds so pool threads are not left blocked on an empty queue.
            let main_result = {
                let _shutdown_after_main = ShutdownAfterMain(Arc::clone(&shared));
                let mut main_worker = Worker::new_main(Arc::clone(&shared));
                main_worker.run()
            };

            // Collect pool worker errors. If the main task already failed, prefer that diagnostic
            // and drop pool errors so the caller sees a single primary failure.
            for handle in handles {
                match handle.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) if main_result.is_ok() => return Err(e),
                    Ok(Err(_)) => {}
                    Err(_) if main_result.is_ok() => {
                        return Err(internal_error(
                            "Worker thread panicked",
                            "A background VM worker crashed unexpectedly. This indicates a VM bug.",
                            SourceLocation::new(1, 1),
                        ));
                    }
                    Err(_) => {}
                }
            }

            main_result
        })
    }
}

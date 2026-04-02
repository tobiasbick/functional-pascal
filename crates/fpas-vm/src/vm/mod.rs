//! Parallel stack VM for FPAS bytecode.
//!
//! The VM executes the main program on the calling thread. Tasks created with
//! `go` are distributed across a thread pool for true parallel execution.
//!
//! **Documentation:** `docs/future/parallel-vm.md`, `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, SourceLocation};
use fpas_std::{Console, ConsoleEvent, ConsoleKeyEvent, KeyInput, TextInput};
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Condvar, Mutex, RwLock};

mod diagnostics;
mod execute;
mod helpers;
mod shared;
mod worker;

pub use diagnostics::VmError;
pub(crate) use diagnostics::{internal_error, runtime_error};
pub(crate) use shared::{SharedState, TaskResultPoll, TaskState};
pub(crate) use worker::Worker;

const STACK_MAX: usize = 4096;
const TIMESLICE: u32 = 256;

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
        Self::build(chunk, Console::new())
    }

    /// Create a VM that streams output to the given writer immediately.
    pub fn with_writer(chunk: Chunk, writer: Box<dyn Write + Send>) -> Self {
        Self::build(chunk, Console::with_writer(writer))
    }

    fn build(chunk: Chunk, console: Console) -> Self {
        let pool_size = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .saturating_sub(1)
            .max(1); // always keep one pool worker so `go` can make progress on single-core hosts

        let shared = Arc::new(SharedState {
            chunk,
            globals: RwLock::new(HashMap::new()),
            task_queue: Mutex::new(Vec::new()),
            task_available: Condvar::new(),
            task_results: Mutex::new(HashMap::new()),
            next_task_id: AtomicU64::new(1),
            console: Mutex::new(console),
            text_input: Mutex::new(TextInput::new()),
            key_input: Mutex::new(KeyInput::new()),
            shutdown: AtomicBool::new(false),
        });

        Self { shared, pool_size }
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

    /// Access captured output (for test assertions).
    pub fn output(&self) -> VmOutput {
        self.shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .clone()
    }

    /// Execute the loaded program.
    ///
    /// The main program runs on the calling thread. If `go` tasks are spawned,
    /// a thread pool is created to execute them in parallel.
    pub fn run(&mut self) -> Result<(), VmError> {
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

            // Run main program on this thread.
            let mut main_worker = Worker::new_main(Arc::clone(&shared));
            let main_result = main_worker.run();

            // Main task done — signal pool to shut down.
            shared.request_shutdown();

            // Collect pool worker errors.
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

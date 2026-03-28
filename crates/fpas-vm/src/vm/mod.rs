//! Stack VM for FPAS bytecode. Handles `Std.Console` input intrinsics and `Std.Array` `Push`/`Pop` opcodes.
//!
//! **Documentation:** `docs/pascal/std/console.md`, `docs/pascal/std/array.md` (from the repository root).
//! **Maintenance:** Keep those Markdown files in sync when changing console I/O or array local op behavior here.

use fpas_bytecode::{Chunk, SourceLocation, Value};
use fpas_std::{Console, ConsoleEvent, ConsoleKeyEvent, KeyInput, TextInput};
use std::collections::{HashMap, VecDeque};
use std::io::Write;

mod diagnostics;
mod execute;
mod helpers;

pub use diagnostics::VmError;
pub(crate) use diagnostics::{internal_error, runtime_error};

const STACK_MAX: usize = 4096;
const TIMESLICE: u32 = 256;

/// Re-export captured output type from fpas-std.
pub type VmOutput = fpas_std::CapturedOutput;

/// Call frame for function invocations.
#[derive(Debug)]
struct CallFrame {
    /// Return address (instruction pointer to resume after call).
    return_ip: usize,
    /// Base slot of this frame on the value stack.
    base_slot: usize,
}

/// Saved state of a suspended task.
struct TaskContext {
    id: u64,
    ip: usize,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
}

/// Buffered channel state.
struct ChannelState {
    buffer: VecDeque<Value>,
    capacity: usize,
    closed: bool,
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    current_location: SourceLocation,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    console: Console,
    /// Line-buffered stdin for `Read` / `ReadLn` (test lines via `push_readln_input`).
    text_input: TextInput,
    /// CRT-style keyboard buffer for `ReadKey` / `KeyPressed` (tests via `push_readkey_input`).
    key_input: KeyInput,
    // -- concurrency --
    tasks: Vec<TaskContext>,
    current_task_id: u64,
    next_task_id: u64,
    channels: HashMap<u64, ChannelState>,
    next_channel_id: u64,
    task_results: HashMap<u64, Value>,
    instructions_until_yield: u32,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            current_location: SourceLocation::new(1, 1),
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            console: Console::new(),
            text_input: TextInput::new(),
            key_input: KeyInput::new(),
            tasks: Vec::new(),
            current_task_id: 0,
            next_task_id: 1,
            channels: HashMap::new(),
            next_channel_id: 1,
            task_results: HashMap::new(),
            instructions_until_yield: TIMESLICE,
        }
    }

    /// Create a VM that streams output to the given writer immediately.
    pub fn with_writer(chunk: Chunk, writer: Box<dyn Write>) -> Self {
        Self {
            chunk,
            ip: 0,
            current_location: SourceLocation::new(1, 1),
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            console: Console::with_writer(writer),
            text_input: TextInput::new(),
            key_input: KeyInput::new(),
            tasks: Vec::new(),
            current_task_id: 0,
            next_task_id: 1,
            channels: HashMap::new(),
            next_channel_id: 1,
            task_results: HashMap::new(),
            instructions_until_yield: TIMESLICE,
        }
    }

    /// Queue a line for the next line-buffered `Read` / `ReadLn` (tests).
    pub fn push_readln_input(&mut self, line: &str) {
        self.text_input.push_line(line);
    }

    /// Queue characters for the next `Std.Console.ReadKey` calls (tests).
    pub fn push_readkey_input(&mut self, s: &str) {
        self.key_input.push_chars(s);
    }

    /// Queue a structured key for the next `Std.Console.ReadKeyEvent` (tests).
    pub fn push_key_event(&mut self, ev: ConsoleKeyEvent) {
        self.key_input.push_key_event(ev);
    }

    /// Queue a structured console event for the next `Std.Console.ReadEvent` (tests).
    pub fn push_console_event(&mut self, ev: ConsoleEvent) {
        self.key_input.push_console_event(ev);
    }

    pub fn output(&self) -> &VmOutput {
        self.console.output()
    }
}

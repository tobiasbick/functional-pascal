use crate::Vm;
use crate::vm::{GraphState, SharedState, TaskTimers};
use fpas_bytecode::{Chunk, Op, SourceLocation, Value};
use fpas_std::{Console, KeyInput, TextInput};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Condvar, Mutex, RwLock};

/// Minimal [`SharedState`] shell for tests (matches [`crate::vm::Vm::build`] field init except chunk).
pub(crate) fn minimal_shared_state(chunk: Chunk) -> SharedState {
    SharedState {
        chunk: Arc::new(chunk),
        program_args: Vec::new(),
        globals: RwLock::new(HashMap::new()),
        task_queue: Mutex::new(VecDeque::new()),
        task_available: Condvar::new(),
        task_timers: TaskTimers::new(),
        task_results: Mutex::new(HashMap::new()),
        task_completions: Mutex::new(HashSet::new()),
        task_results_available: Condvar::new(),
        completed_task_count: AtomicU64::new(0),
        next_task_id: AtomicU64::new(1),
        console: Mutex::new(Console::new()),
        text_input: Mutex::new(TextInput::new()),
        key_input: Mutex::new(KeyInput::new()),
        graph: Mutex::new(GraphState::default()),
        shutdown: AtomicBool::new(false),
        abort_spawned_bytecode: AtomicBool::new(false),
    }
}

pub(crate) fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

pub(super) fn graph_application_value() -> Value {
    Value::record("Std.Graph.Application".into(), vec![])
}

pub(super) fn graph_size_value(width: i64, height: i64) -> Value {
    Value::record(
        "Std.Graph.Size".into(),
        vec![
            ("width".into(), Value::Integer(width)),
            ("height".into(), Value::Integer(height)),
        ],
    )
}

pub(super) fn emit_constant(chunk: &mut Chunk, value: Value) {
    let idx = chunk
        .add_constant(value)
        .expect("constant should fit in test chunk");
    chunk.emit(Op::Constant(idx), loc());
}

pub(super) fn build_function_chunk(
    function_name: &str,
    arity: u8,
    main: impl FnOnce(&mut Chunk),
    body: impl FnOnce(&mut Chunk),
) -> Chunk {
    let mut chunk = Chunk::new();
    main(&mut chunk);
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk.insert_function(function_name.to_string(), code_start, arity);
    body(&mut chunk);
    chunk
}

pub(super) fn build_zero_arg_function_chunk(
    function_name: &str,
    main: impl FnOnce(&mut Chunk),
    body: impl FnOnce(&mut Chunk),
) -> Chunk {
    build_function_chunk(function_name, 0, main, body)
}

pub(super) fn run_err(chunk: Chunk) -> fpas_diagnostics::Diagnostic {
    let mut vm = Vm::new(chunk);
    vm.run().expect_err("VM should return an error")
}

pub(super) fn run_ok_output(chunk: Chunk) -> Vec<String> {
    let mut vm = Vm::new(chunk);
    vm.run().expect("VM should succeed");
    vm.output().lines
}

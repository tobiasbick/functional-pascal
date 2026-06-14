use crate::Vm;
use crate::vm::{GraphState, SharedState};
use fpas_bytecode::{Chunk, Op, SourceLocation, Value};
use fpas_std::{Console, ConsoleEvent, ConsoleKeyEvent, KeyInput, TextInput};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Condvar, Mutex, RwLock};

/// Minimal [`SharedState`] shell for tests (matches [`crate::vm::Vm::build`] field init except chunk).
pub(super) fn minimal_shared_state(chunk: Chunk) -> SharedState {
    SharedState {
        chunk,
        program_args: Vec::new(),
        globals: RwLock::new(HashMap::new()),
        task_queue: Mutex::new(Vec::new()),
        task_available: Condvar::new(),
        task_results: Mutex::new(HashMap::new()),
        task_results_available: Condvar::new(),
        next_task_id: AtomicU64::new(1),
        console: Mutex::new(Console::new()),
        text_input: Mutex::new(TextInput::new()),
        key_input: Mutex::new(KeyInput::new()),
        tui: Mutex::new(Default::default()),
        graph: Mutex::new(GraphState::default()),
        shutdown: AtomicBool::new(false),
        abort_spawned_bytecode: AtomicBool::new(false),
    }
}

pub(super) fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

pub(super) fn tui_application_value() -> Value {
    Value::Record {
        type_name: "Std.Tui.Application".into(),
        fields: vec![],
    }
}

pub(super) fn graph_application_value() -> Value {
    Value::Record {
        type_name: "Std.Graph.Application".into(),
        fields: vec![],
    }
}

pub(super) fn graph_size_value(width: i64, height: i64) -> Value {
    Value::Record {
        type_name: "Std.Graph.Size".into(),
        fields: vec![
            ("width".into(), Value::Integer(width)),
            ("height".into(), Value::Integer(height)),
        ],
    }
}

pub(super) fn key_event_value(ev: ConsoleKeyEvent) -> Value {
    Value::Record {
        type_name: "Std.Console.KeyEvent".into(),
        fields: vec![
            ("kind".into(), Value::Integer(ev.kind as i64)),
            ("ch".into(), Value::Char(ev.ch)),
            ("shift".into(), Value::Boolean(ev.shift)),
            ("ctrl".into(), Value::Boolean(ev.ctrl)),
            ("alt".into(), Value::Boolean(ev.alt)),
            ("meta".into(), Value::Boolean(ev.meta)),
        ],
    }
}

/// Builds a `Std.Console.Event` record for bytecode-level injection tests.
pub(super) fn console_event_value(event: ConsoleEvent) -> Value {
    Value::Record {
        type_name: "Std.Console.Event".into(),
        fields: vec![
            ("kind".into(), Value::Integer(event.kind as i64)),
            ("key".into(), key_event_value(event.key)),
            (
                "mouse_action".into(),
                Value::Integer(event.mouse_action as i64),
            ),
            (
                "mouse_button".into(),
                Value::Integer(event.mouse_button as i64),
            ),
            ("mouse_x".into(), Value::Integer(event.mouse_x)),
            ("mouse_y".into(), Value::Integer(event.mouse_y)),
            ("width".into(), Value::Integer(event.width)),
            ("height".into(), Value::Integer(event.height)),
            ("text".into(), Value::Str(event.text)),
            ("shift".into(), Value::Boolean(event.shift)),
            ("ctrl".into(), Value::Boolean(event.ctrl)),
            ("alt".into(), Value::Boolean(event.alt)),
            ("meta".into(), Value::Boolean(event.meta)),
        ],
    }
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
    chunk
        .functions
        .insert(function_name.to_string(), (code_start, arity));
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

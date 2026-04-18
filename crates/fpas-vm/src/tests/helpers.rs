use crate::Vm;
use crate::vm::SharedState;
use fpas_bytecode::{Chunk, Op, SourceLocation, Value};
use fpas_std::{Console, ConsoleKeyEvent, KeyInput, TextInput};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Condvar, Mutex, RwLock};

/// Minimal [`SharedState`] shell for tests (matches [`crate::vm::Vm::build`] field init except chunk).
pub(super) fn minimal_shared_state(chunk: Chunk) -> SharedState {
    SharedState {
        chunk,
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

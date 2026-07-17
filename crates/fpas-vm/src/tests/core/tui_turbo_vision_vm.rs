//! VM regression tests for the internal Turbo Vision `Std.Tui` bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (current callback contract).

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};
use crate::vm::Worker;
use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use std::sync::Arc;

fn emit_open_for_test(chunk: &mut Chunk, width: i64, height: i64) {
    emit_constant(chunk, Value::Integer(width));
    emit_constant(chunk, Value::Integer(height));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
}

fn tui_rect_value(x: i64, y: i64, width: i64, height: i64) -> Value {
    Value::Record {
        type_name: "Std.Tui.TuiRect".into(),
        fields: vec![
            ("x".into(), Value::Integer(x)),
            ("y".into(), Value::Integer(y)),
            ("width".into(), Value::Integer(width)),
            ("height".into(), Value::Integer(height)),
        ],
    }
}

#[test]
fn turbo_vision_command_event_dispatches_registered_fpas_on_command() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_command_start = chunk.len();
    chunk.insert_function("OnCommand", on_command_start, 2);
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("callback registration should succeed");

    worker
        .bridge_dispatch_command_event_for_tests(42, loc())
        .expect("command dispatch should succeed");

    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec!["42"]);
}

#[test]
fn turbo_vision_input_line_text_read_back_via_input_line_text() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_constant(&mut chunk, tui_rect_value(12, 2, 20, 1));
    emit_constant(&mut chunk, Value::Str("initial".into()));
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::InputLineNew))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::InputLineText))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("input line read-back should succeed");

    assert_eq!(worker.stack.last(), Some(&Value::Str("initial".into())));
}

/// `Application.Quit` sets quit flags when dispatched from a registered `OnCommand` handler.
#[test]
fn turbo_vision_on_command_handler_can_quit_application() {
    use fpas_std::CM_QUIT;

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_command_start = chunk.len();
    chunk.insert_function("OnCommand", on_command_start, 2);
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::Quit))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("OnCommand registration should succeed");

    worker
        .bridge_dispatch_command_event_for_tests(CM_QUIT as u16, loc())
        .expect("command dispatch should succeed");

    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec![CM_QUIT.to_string()]);
    let quit = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(quit.quit_requested);
}

#[test]
fn turbo_vision_keyboard_event_maps_escape_kind() {
    use fpas_std::key_event::key_kind_index;
    use turbo_vision::core::event::{Event as TurboVisionEvent, KB_ESC};

    let event = TurboVisionEvent::keyboard(KB_ESC);
    let key = Worker::turbo_vision_keyboard_to_console_key_for_tests(&event);
    assert_eq!(key.kind, key_kind_index("Escape"));
}

#[test]
fn turbo_vision_unhandled_keyboard_dispatches_on_key() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::RegisterOnKey))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.insert_function("OnKey", on_key_start, 2);
    emit_constant(&mut chunk, Value::Str("key".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("OnKey registration should succeed");

    let mut event = turbo_vision::core::event::Event::keyboard(turbo_vision::core::event::KB_ESC);
    worker
        .dispatch_turbo_vision_unhandled_input_for_tests(&mut event, loc())
        .expect("OnKey dispatch should succeed");

    assert_eq!(event.what, turbo_vision::core::event::EventType::Nothing);
    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec!["key"]);
}

#[test]
fn turbo_vision_unhandled_keyboard_dispatches_on_key_and_can_quit() {
    use turbo_vision::core::event::{Event as TurboVisionEvent, KB_ESC};

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
            task_bound: false,
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::RegisterOnKey))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.insert_function("OnKey", on_key_start, 2);
    emit_constant(&mut chunk, Value::Str("key".into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::Quit))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Boolean(false));
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("OnKey registration should succeed");

    let mut event = TurboVisionEvent::keyboard(KB_ESC);
    worker
        .dispatch_turbo_vision_unhandled_input_for_tests(&mut event, loc())
        .expect("OnKey dispatch should succeed");

    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec!["key"]);
}

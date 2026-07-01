//! VM regression tests for the internal Turbo Vision `Std.Tui` bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (current callback contract).

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};
use crate::vm::Worker;
use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ProcessOutcome;
use std::sync::Arc;
use turbo_vision::core::event::Event as TurboVisionEvent;

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
        type_name: "Std.Tui.Rect".into(),
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
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnCommand,
        ))),
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

    let outcome = worker
        .dispatch_turbo_vision_command_event_for_tests(&TurboVisionEvent::command(42), loc())
        .expect("command dispatch should succeed");

    assert_eq!(outcome, Some(ProcessOutcome::Command { handled: true }));
    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec!["42"]);
}

#[test]
fn turbo_vision_input_line_text_read_back_via_input_text() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, tui_rect_value(12, 2, 20, 1));
    emit_constant(&mut chunk, Value::Str("initial".into()));
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::CreateInputLine))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::InputText))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("input line read-back should succeed");

    assert_eq!(worker.stack.last(), Some(&Value::Str("initial".into())));
}

/// `Application.Run` selects the Turbo Vision backend when any widget handle exists,
/// so `OnPaint` is not required on that path.
#[test]
fn application_run_uses_turbo_vision_path_when_widget_handles_exist() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 40, 12);
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, tui_rect_value(2, 1, 24, 8));
    emit_constant(&mut chunk, Value::Str("Demo".into()));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::CreateDialog))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker
        .run()
        .expect("turbo vision Application.Run should not require OnPaint");
}

/// Reserved Turbo Vision built-in command ids are translated back before `OnCommand`.
#[test]
fn turbo_vision_dispatch_translates_offset_reserved_command_to_fpas_id() {
    const FPAS_TV_COMMAND_OFFSET: u16 = 0x8000;

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
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnCommand,
        ))),
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

    let tv_command = 24u16 + FPAS_TV_COMMAND_OFFSET;
    let outcome = worker
        .dispatch_turbo_vision_command_event_for_tests(
            &TurboVisionEvent::command(tv_command),
            loc(),
        )
        .expect("command dispatch should succeed");

    assert_eq!(outcome, Some(ProcessOutcome::Command { handled: true }));
    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert_eq!(output.lines, vec!["24"]);
}

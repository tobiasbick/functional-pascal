//! VM tests for `Std.Tui.Application.Configure`.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use std::sync::Arc;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state, run_err};
use crate::vm::Worker;

fn handlers_record(fields: Vec<(String, Value)>) -> Value {
    Value::Record {
        type_name: "Std.Tui.ApplicationHandlers".into(),
        fields,
    }
}

fn function_value(name: &str) -> Value {
    Value::Function {
        name: name.into(),
        captures: vec![],
    }
}

#[test]
fn tui_application_configure_stores_bundle_handlers_and_interval() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        handlers_record(vec![
            ("OnPaint".into(), function_value("OnPaint")),
            (
                "OnKeyPressed".into(),
                Value::OptionSome(Box::new(function_value("OnKeyPressed"))),
            ),
            ("OnResize".into(), Value::OptionNone),
            ("OnIdleMilliseconds".into(), Value::Integer(12)),
            (
                "OnIdle".into(),
                Value::OptionSome(Box::new(function_value("OnIdle"))),
            ),
            (
                "OnExit".into(),
                Value::OptionSome(Box::new(function_value("OnExit"))),
            ),
            ("OnMouse".into(), Value::OptionNone),
            ("OnPaste".into(), Value::OptionNone),
            ("OnFocusGained".into(), Value::OptionNone),
            ("OnFocusLost".into(), Value::OptionNone),
            ("OnActivate".into(), Value::OptionNone),
            ("OnDeactivate".into(), Value::OptionNone),
        ]),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationConfigure as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_key_start = chunk.len();
    chunk
        .functions
        .insert("OnKeyPressed".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(matches!(
        tui.on_paint.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnPaint"
    ));
    assert!(matches!(
        tui.on_key_pressed.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnKeyPressed"
    ));
    assert!(tui.on_resize.is_none(), "OnResize should stay cleared");
    assert_eq!(tui.idle_interval_ms, 12);
    assert!(matches!(
        tui.on_idle.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnIdle"
    ));
    assert!(matches!(
        tui.on_exit.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnExit"
    ));
}

#[test]
fn tui_application_configure_clears_previous_optional_handlers_with_none_defaults() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, function_value("OldOnExit"));
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    emit_constant(
        &mut chunk,
        handlers_record(vec![
            ("OnPaint".into(), function_value("OnPaint")),
            ("OnKeyPressed".into(), Value::OptionNone),
            ("OnResize".into(), Value::OptionNone),
            ("OnIdleMilliseconds".into(), Value::Integer(0)),
            ("OnIdle".into(), Value::OptionNone),
            ("OnExit".into(), Value::OptionNone),
            ("OnMouse".into(), Value::OptionNone),
            ("OnPaste".into(), Value::OptionNone),
            ("OnFocusGained".into(), Value::OptionNone),
            ("OnFocusLost".into(), Value::OptionNone),
            ("OnActivate".into(), Value::OptionNone),
            ("OnDeactivate".into(), Value::OptionNone),
        ]),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationConfigure as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk
        .functions
        .insert("OldOnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_exit.is_none(),
        "Application.Configure should clear OnExit"
    );
    assert!(
        tui.on_idle.is_none(),
        "Application.Configure should clear OnIdle"
    );
    assert_eq!(tui.idle_interval_ms, 0);
}

#[test]
fn tui_application_configure_rejects_missing_required_on_paint_field() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        handlers_record(vec![
            ("OnKeyPressed".into(), Value::OptionNone),
            ("OnResize".into(), Value::OptionNone),
            ("OnIdleMilliseconds".into(), Value::Integer(0)),
            ("OnIdle".into(), Value::OptionNone),
            ("OnExit".into(), Value::OptionNone),
        ]),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationConfigure as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("missing field `OnPaint`"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_application_configure_rejects_non_option_optional_handler_value() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        handlers_record(vec![
            ("OnPaint".into(), function_value("OnPaint")),
            ("OnKeyPressed".into(), Value::Integer(7)),
            ("OnResize".into(), Value::OptionNone),
            ("OnIdleMilliseconds".into(), Value::Integer(0)),
            ("OnIdle".into(), Value::OptionNone),
            ("OnExit".into(), Value::OptionNone),
        ]),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationConfigure as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("ApplicationHandlers.OnKeyPressed must be `Some(handler)` or `None`"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_application_configure_clamps_negative_idle_interval_to_zero() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        handlers_record(vec![
            ("OnPaint".into(), function_value("OnPaint")),
            ("OnKeyPressed".into(), Value::OptionNone),
            ("OnResize".into(), Value::OptionNone),
            ("OnIdleMilliseconds".into(), Value::Integer(-25)),
            (
                "OnIdle".into(),
                Value::OptionSome(Box::new(function_value("OnIdle"))),
            ),
            ("OnExit".into(), Value::OptionNone),
            ("OnMouse".into(), Value::OptionNone),
            ("OnPaste".into(), Value::OptionNone),
            ("OnFocusGained".into(), Value::OptionNone),
            ("OnFocusLost".into(), Value::OptionNone),
            ("OnActivate".into(), Value::OptionNone),
            ("OnDeactivate".into(), Value::OptionNone),
        ]),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationConfigure as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.idle_interval_ms, 0);
}

//! VM bridge for `TuiHost` (Phase 3): host poll/register/process intrinsics (see `docs/pascal/std/tui-app.md`).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_std::ConsoleEvent;
use fpas_std::ConsoleKeyEvent;
use fpas_std::key_event::key_kind_index;
use std::sync::Arc;

use crate::Vm;
use crate::tests::helpers::{
    emit_constant, key_event_value, loc, minimal_shared_state, run_err, run_ok_output,
    tui_application_value,
};
use crate::vm::Worker;

#[test]
fn tui_host_invoke_on_key_pressed_runs_registered_fp_function() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnKeyPressed as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(
        &mut chunk,
        key_event_value(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )),
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostInvokeOnKeyPressed as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.functions.insert("OnKey".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["true"]);
}

#[test]
fn tui_host_poll_next_coalesces_resize_before_key() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPollNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPollNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPollNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(10, 10));
    vm.push_console_event(ConsoleEvent::resize(30, 20));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    let lines = vm.output().lines;
    assert_eq!(lines[0], "None", "resize-only poll buffers coalesced size");
    assert_eq!(
        lines[1], "None",
        "second resize still waits for a key before flush"
    );
    assert!(
        lines[2].contains("30"),
        "third poll (key) flushes coalesced resize (width 30): {}",
        lines[2]
    );
}

#[test]
fn tui_host_process_next_dispatches_on_resize_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnResize as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_resize_start = chunk.len();
    chunk
        .functions
        .insert("OnResize".into(), (on_resize_start, 2));
    emit_constant(&mut chunk, Value::Str("r".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(10, 10));
    vm.push_console_event(ConsoleEvent::resize(30, 20));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["r", "2"]);
}

#[test]
fn tui_host_process_next_resize_without_handler_returns_tag_four() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(5, 5));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("A"),
        'a',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["4"]);
}

#[test]
fn tui_host_dispatch_redraw_invokes_on_paint() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationRequestRedraw as u16),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaint as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostDispatchRedraw as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["p", "5"]);
}

#[test]
fn tui_host_register_on_idle_stores_handler_and_interval() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(25));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnIdle as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(matches!(
        tui.on_idle.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnIdle"
    ));
    assert_eq!(tui.idle_interval_ms, 25);
}

#[test]
fn tui_host_register_on_idle_clamps_negative_interval_to_zero() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(-5));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnIdle as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

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

#[test]
fn tui_host_dispatch_redraw_without_handler_clears_and_returns_six() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationRequestRedraw as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostDispatchRedraw as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["6"]);
}

#[test]
fn tui_host_dispatch_redraw_when_not_pending_returns_zero() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostDispatchRedraw as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["0"]);
}

#[test]
fn tui_host_run_loop_dispatches_paint_then_key_until_idle() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationRequestRedraw as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnKeyPressed as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaint as u16),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(16));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRunLoop as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.functions.insert("OnKey".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["p"]);
}

#[test]
fn tui_host_request_quit_ends_host_run_loop_after_idle_iteration() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRequestQuit as u16), loc());
    emit_constant(&mut chunk, Value::Integer(10_000));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRunLoop as u16), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert!(
        vm.output().lines.is_empty(),
        "run loop should exit on quit after first idle iteration without printing"
    );
}

#[test]
fn tui_host_run_loop_max_iterations_zero_skips_body() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiApplicationRequestRedraw as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaint as u16),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRunLoop as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert!(run_ok_output(chunk).is_empty());
}

#[test]
fn tui_host_register_on_exit_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(matches!(
        tui.on_exit.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnExit"
    ));
}

#[test]
fn tui_host_register_on_idle_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnIdle as u16),
        loc(),
    );
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_idle.is_none(),
        "Application.Close should clear OnIdle"
    );
    assert_eq!(tui.idle_interval_ms, 0);
}

#[test]
fn tui_host_register_on_idle_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnIdle as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnIdle expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_idle_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnIdle as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnIdle".into(), (on_idle_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnIdle handler must have arity 1"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_exit_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_exit.is_none(),
        "Application.Close should clear OnExit"
    );
}

#[test]
fn tui_host_register_on_exit_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnExit expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_exit_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnExit".into(), (on_exit_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnExit handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_mouse_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnMouse as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_mouse.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnMouse"
        ),
        "on_mouse should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_mouse_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnMouse as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Str("m".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    use fpas_std::{mouse_action_index, mouse_button_index};
    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::mouse(
        mouse_action_index("ScrollUp"),
        mouse_button_index("None"),
        5,
        3,
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["m", "5"]);
}

#[test]
fn tui_host_process_next_mouse_without_handler_returns_tag_seven() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    use fpas_std::{mouse_action_index, mouse_button_index};
    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::mouse(
        mouse_action_index("Down"),
        mouse_button_index("Left"),
        1,
        1,
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["7"]);
}

#[test]
fn tui_host_register_on_mouse_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnMouse as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_mouse.is_none(),
        "on_mouse should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_mouse_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnMouse as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnMouse expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_mouse_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnMouse as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnMouse".into(), (on_mouse_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnMouse handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

// ── OnPaste ──────────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_paste_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaste as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_paste.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnPaste"
        ),
        "on_paste should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_paste_handler_returns_tag_eight() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaste as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Str("paste".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::paste("hello".to_string()));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["paste", "8"]);
}

#[test]
fn tui_host_process_next_paste_without_handler_returns_tag_nine() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::paste("world".to_string()));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["9"]);
}

#[test]
fn tui_host_register_on_paste_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaste as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_paste.is_none(),
        "on_paste should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_paste_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaste as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnPaste expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_paste_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnPaste as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnPaste".into(), (on_paste_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnPaste handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

// ── OnFocusGained ─────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_focus_gained_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusGained as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_focus_gained.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnFocusGained"
        ),
        "on_focus_gained should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_focus_gained_handler_returns_tag_ten() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusGained as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Str("fg".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_gained());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["fg", "10"]);
}

#[test]
fn tui_host_process_next_focus_gained_without_handler_returns_tag_eleven() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_gained());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["11"]);
}

#[test]
fn tui_host_register_on_focus_gained_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusGained as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_focus_gained.is_none(),
        "on_focus_gained should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_focus_gained_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFG".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusGained as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk.functions.insert("WrongFG".into(), (on_fg_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("OnFocusGained handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

// ── OnFocusLost ───────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_focus_lost_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusLost as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_focus_lost.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnFocusLost"
        ),
        "on_focus_lost should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_focus_lost_handler_returns_tag_twelve() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusLost as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
    emit_constant(&mut chunk, Value::Str("fl".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_lost());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["fl", "12"]);
}

#[test]
fn tui_host_process_next_focus_lost_without_handler_returns_tag_thirteen() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_lost());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["13"]);
}

#[test]
fn tui_host_register_on_focus_lost_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusLost as u16),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationClose as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_focus_lost.is_none(),
        "on_focus_lost should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_focus_lost_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFL".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnFocusLost as u16),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk.functions.insert("WrongFL".into(), (on_fl_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("OnFocusLost handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

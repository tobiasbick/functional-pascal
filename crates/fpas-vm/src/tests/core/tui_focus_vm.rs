//! VM-level tests for Phase 7 Step 2: host-managed focus chain, Tab/Shift+Tab traversal,
//! and `OnActivate`/`OnDeactivate` dispatch.
//!
//! These tests populate `TuiState.views` directly from Rust where convenient, and also cover
//! the additive FPAS-facing host view API introduced after the initial focus-chain step.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, ViewRect, key_event::key_kind_index};
use std::sync::Arc;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};
use crate::vm::Worker;

fn tab_event(shift: bool) -> ConsoleEvent {
    ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Tab"),
        '\t',
        shift,
        false,
        false,
        false,
    ))
}

fn view_rect() -> ViewRect {
    ViewRect {
        x: 0,
        y: 0,
        width: 10,
        height: 5,
    }
}

// ---------------------------------------------------------------------------
// Helper: build a chunk that opens TUI, registers handlers, calls
// TuiHostProcessNext once, and halts.
// ---------------------------------------------------------------------------

fn build_process_next_chunk_with_handlers(
    on_activate_name: Option<&str>,
    on_deactivate_name: Option<&str>,
    on_key_name: Option<&str>,
) -> Chunk {
    let mut chunk = Chunk::new();

    // Open TUI.
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());

    if let Some(name) = on_activate_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(Intrinsic::TuiHostRegisterOnActivate as u16),
            loc(),
        );
    }
    if let Some(name) = on_deactivate_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(Intrinsic::TuiHostRegisterOnDeactivate as u16),
            loc(),
        );
    }
    if let Some(name) = on_key_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(Intrinsic::TuiHostRegisterOnKeyPressed as u16),
            loc(),
        );
    }

    // TuiHostProcessNext(App, 64)
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc()); // print the tag
    chunk.emit(Op::Halt, loc());

    chunk
}

fn add_handler(chunk: &mut Chunk, name: &str, arity: u8, body_output: &str) {
    let start = chunk.len();
    chunk.functions.insert(name.to_string(), (start, arity));
    emit_constant(chunk, Value::Str(body_output.into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(chunk, Value::Unit);
    chunk.emit(Op::Return, loc());
}

fn add_key_handler(chunk: &mut Chunk, name: &str, output: &str) {
    let start = chunk.len();
    chunk.functions.insert(name.to_string(), (start, 2)); // (App, Key)
    emit_constant(chunk, Value::Str(output.into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn host_register_view_returns_distinct_integer_handles() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRegisterView as u16), loc());
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRegisterView as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["0", "1"]);
}

#[test]
fn host_push_child_view_populates_focus_chain_and_query_focused_view_id() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRegisterView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPushChildView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRegisterView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPushChildView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostQueryFocusedViewId as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["14", "0"]);
}

#[test]
fn host_unregister_view_removes_it_from_focus_chain() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRegisterView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostPushChildView as u16), loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostUnregisterView as u16),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostQueryFocusedViewId as u16),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["-1"]);

    let tui = shared.tui.lock().unwrap();
    assert!(!tui.views.has_focusable_children());
}

#[test]
fn tab_with_two_focusable_views_fires_on_activate_and_returns_tag_14() {
    let mut chunk =
        build_process_next_chunk_with_handlers(Some("OnActivate"), Some("OnDeactivate"), None);
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_handler(&mut chunk, "OnDeactivate", 1, "deactivate");

    let shared = Arc::new(minimal_shared_state(chunk));

    // Populate the focus chain from Rust: two views.
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    // Inject Tab.
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();

    // Tag 14 = focus advanced; OnDeactivate does NOT fire (no previous focus).
    // OnActivate fires for the newly focused view.
    assert_eq!(lines, vec!["activate", "14"]);
}

#[test]
fn tab_second_press_fires_deactivate_then_activate() {
    // Build chunk that calls TuiHostProcessNext twice.
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnActivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnActivate as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnDeactivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnDeactivate as u16),
        loc(),
    );

    // First TuiHostProcessNext (Dup App so it survives for the second call).
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::Pop, loc()); // discard tag

    // Second TuiHostProcessNext (App still on stack from Dup above).
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostProcessNext as u16), loc());
    chunk.emit(Op::Pop, loc());

    chunk.emit(Op::Halt, loc());

    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_handler(&mut chunk, "OnDeactivate", 1, "deactivate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    // Two Tab events.
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    // First Tab: no previous focus → only OnActivate fires.
    // Second Tab: previous focus exists → OnDeactivate then OnActivate fires.
    assert_eq!(lines, vec!["activate", "deactivate", "activate"]);
}

#[test]
fn shift_tab_fires_on_activate_and_returns_tag_15() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, None);
    add_handler(&mut chunk, "OnActivate", 1, "activate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(true));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["activate", "15"]);
}

#[test]
fn tab_with_no_focusable_views_dispatches_to_on_key_pressed() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    // No push_child calls → empty focus chain.

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    // OnActivate must NOT fire; OnKey fires; tag = 1 (key dispatched).
    assert_eq!(lines, vec!["key", "1"]);
}

#[test]
fn tab_with_single_unfocused_view_establishes_focus_fires_on_activate() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        tui.views.push_child(a);
        // focused = None → first Tab should establish focus.
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    // Single child, not yet focused → OnActivate fires; tag = 14.
    assert_eq!(lines, vec!["activate", "14"]);
}

#[test]
fn tab_with_single_already_focused_view_falls_through_to_on_key_pressed() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.focus_next(); // establish focus before the event
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    // Already focused on the only view → Tab falls through to OnKeyPressed.
    assert_eq!(lines, vec!["key", "1"]);
}

#[test]
fn close_after_run_clears_on_activate_and_on_deactivate() {
    // Check that close_tui_application_state resets the new handlers.
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
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
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnActivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnActivate as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnDeactivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnDeactivate as u16),
        loc(),
    );
    // HostRequestQuit so Application.Run exits immediately.
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRequestQuit as u16), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationRun as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_activate_start = chunk.len();
    chunk
        .functions
        .insert("OnActivate".into(), (on_activate_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_deactivate_start = chunk.len();
    chunk
        .functions
        .insert("OnDeactivate".into(), (on_deactivate_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap();
    assert!(
        tui.on_activate.is_none(),
        "Application.Run should clear on_activate"
    );
    assert!(
        tui.on_deactivate.is_none(),
        "Application.Run should clear on_deactivate"
    );
}

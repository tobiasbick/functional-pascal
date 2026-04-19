//! VM tests for `Std.Tui.Application.Run`.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use std::sync::Arc;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state, run_err};
use crate::vm::Worker;

#[test]
fn tui_application_run_invokes_on_exit_and_clears_shared_state() {
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
            name: "OnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(Intrinsic::TuiHostRegisterOnExit as u16),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiHostRequestQuit as u16), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationRun as u16), loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Str("x".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    assert_eq!(
        worker
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .lines
            .clone(),
        vec!["p", "x"],
    );

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_paint.is_none(),
        "Application.Run should clear OnPaint"
    );
    assert!(tui.on_exit.is_none(), "Application.Run should clear OnExit");
    assert!(
        tui.last_exit_reason.is_none(),
        "Application.Run close semantics should clear the last exit reason"
    );
    assert!(
        !tui.run_active,
        "Application.Run should reset the active-run guard"
    );
}

#[test]
fn tui_application_run_rejects_missing_on_paint_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationOpen as u16), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TuiApplicationRun as u16), loc());
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("Application.Run(App) requires a registered OnPaint handler"),
        "unexpected runtime error: {}",
        error.message
    );
}

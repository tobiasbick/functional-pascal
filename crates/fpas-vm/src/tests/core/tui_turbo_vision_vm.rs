//! VM regression tests for the internal Turbo Vision `Std.Tui` bridge.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (current callback contract).

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ProcessOutcome;
use std::sync::Arc;
use turbo_vision::core::event::Event as TurboVisionEvent;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};
use crate::vm::Worker;

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

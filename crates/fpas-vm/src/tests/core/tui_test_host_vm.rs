//! VM tests for headless native TUI testing (Phase 1).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ConsoleEvent;

use crate::Vm;
use crate::tests::helpers::{emit_constant, loc, tui_application_value};

#[test]
fn tui_open_for_test_close_and_reopen_succeeds() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(25));
    emit_constant(&mut chunk, Value::Integer(80));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::CloseForTest))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(40));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::CloseForTest))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Str("ok".into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["ok"]);
}

#[test]
fn tui_test_pump_dispatches_one_resize_event() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(25));
    emit_constant(&mut chunk, Value::Integer(80));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnResize,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
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
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["r"]);
}

#[test]
fn tui_test_pump_until_idle_drains_multiple_events() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(25));
    emit_constant(&mut chunk, Value::Integer(80));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnResize,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPumpUntilIdle))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_resize_start = chunk.len();
    chunk
        .functions
        .insert("OnResize".into(), (on_resize_start, 2));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(10, 10));
    vm.push_console_event(ConsoleEvent::resize(30, 20));
    vm.push_console_event(ConsoleEvent::resize(40, 15));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["1"]);
}

//! VM tests for native TUI input injection (Phase 2).
//!
//! **Documentation:** `docs/pascal/std/tui/app.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_event::key_kind_index};

use crate::Vm;
use crate::tests::helpers::{
    console_event_value, emit_constant, key_event_value, loc, tui_application_value,
};

fn emit_open_for_test(chunk: &mut Chunk) {
    emit_constant(chunk, Value::Integer(25));
    emit_constant(chunk, Value::Integer(80));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
        loc(),
    );
}

#[test]
fn tui_test_send_key_dispatches_on_key_pressed() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnKeyPressed,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(
        &mut chunk,
        key_event_value(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestSendKey))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.functions.insert("OnKey".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Str("k".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["k"]);
}

#[test]
fn tui_test_send_mouse_dispatches_on_mouse_handler() {
    use fpas_std::{mouse_action_index, mouse_button_index};

    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(
        &mut chunk,
        console_event_value(ConsoleEvent::mouse(
            mouse_action_index("Down"),
            mouse_button_index("Left"),
            4,
            2,
            false,
            false,
            false,
            false,
        )),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestSendMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Str("m".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["m"]);
}

#[test]
fn tui_test_move_mouse_dispatches_on_mouse_handler() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestMoveMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Str("move".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["move"]);
}

#[test]
fn tui_test_click_mouse_dispatches_two_mouse_events() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestClickMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPumpUntilIdle))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["1", "1"]);
}

#[test]
fn tui_test_resize_dispatches_on_resize_handler() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
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
    emit_constant(&mut chunk, Value::Integer(30));
    emit_constant(&mut chunk, Value::Integer(100));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestResize))),
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
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["r"]);
}

#[test]
fn tui_test_paste_dispatches_on_paste_handler() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Str("hello".into()));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPaste))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["p"]);
}

#[test]
fn tui_test_focus_dispatches_on_focus_gained_handler() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk);
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusGained,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestFocus))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_focus_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_focus_start, 2));
    emit_constant(&mut chunk, Value::Str("f".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["f"]);
}

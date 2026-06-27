//! VM-level event-ordering tests for hosted `Std.Tui` dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_event::key_kind_index};

use crate::Vm;
use crate::tests::helpers::{emit_constant, loc};

#[test]
fn tui_host_process_next_dispatches_resize_burst_before_key() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );

    chunk.emit(Op::Dup, loc());
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

    chunk.emit(Op::Dup, loc());
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

    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_resize_start = chunk.len();
    chunk.insert_function("OnResize", on_resize_start, 2);
    emit_constant(&mut chunk, Value::Str("resize".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_key_start = chunk.len();
    chunk.insert_function("OnKey", on_key_start, 2);
    emit_constant(&mut chunk, Value::Str("key".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Boolean(true));
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

    vm.run().expect("VM should succeed");

    assert_eq!(vm.output().lines, vec!["resize", "2", "key", "1"]);
}

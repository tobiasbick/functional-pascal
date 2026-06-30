//! VM tests for the public TUI screen query intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use crate::tests::helpers::{
    emit_constant, loc, minimal_shared_state, tui_application_value, tui_screen_cell_value,
    tui_size_value,
};
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

#[test]
fn tui_query_screen_size_returns_open_dimensions() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryScreenSize))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query screen size should succeed");

    assert_eq!(
        worker.stack,
        vec![tui_application_value(), tui_size_value(80, 25)]
    );
}

#[test]
fn tui_query_screen_line_reads_blank_headless_row() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryScreenLine))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query screen line should succeed");

    let Value::Str(line) = worker.stack.last().expect("line on stack") else {
        panic!("expected string line");
    };
    assert_eq!(line.len(), 80);
    assert!(line.chars().all(|ch| ch == ' '), "got {line:?}");
}

#[test]
fn tui_query_screen_cell_reads_blank_headless_cell() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryScreenCell))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query screen cell should succeed");

    assert_eq!(worker.stack.last(), Some(&tui_screen_cell_value(' ', 7, 0)));
}

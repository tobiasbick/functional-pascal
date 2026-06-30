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

fn menu_popup_exit_item() -> Value {
    Value::Record {
        type_name: "Std.Tui.MenuPopupItem".into(),
        fields: vec![
            ("Label".into(), Value::Str("Exit".into())),
            ("Shortcut".into(), Value::Str("X".into())),
            ("Enabled".into(), Value::Boolean(true)),
            ("CommandId".into(), Value::Integer(1)),
            ("Separator".into(), Value::Boolean(false)),
        ],
    }
}

fn menu_bar_file_item() -> Value {
    Value::Record {
        type_name: "Std.Tui.MenuBarItem".into(),
        fields: vec![
            ("Label".into(), Value::Str("File".into())),
            ("Shortcut".into(), Value::Str("F".into())),
            ("Enabled".into(), Value::Boolean(true)),
            ("CommandId".into(), Value::Integer(-1)),
            ("Submenu".into(), Value::Array(vec![menu_popup_exit_item()])),
        ],
    }
}

fn default_menu_bar_style() -> Value {
    Value::Record {
        type_name: "Std.Tui.MenuBarStyle".into(),
        fields: vec![
            ("BarBg".into(), Value::Integer(7)),
            ("BarFg".into(), Value::Integer(0)),
            ("AccelFg".into(), Value::Integer(4)),
            ("HighlightBg".into(), Value::Integer(0)),
            ("HighlightFg".into(), Value::Integer(7)),
            ("DisabledFg".into(), Value::Integer(8)),
        ],
    }
}

fn emit_menu_bar(chunk: &mut Chunk) {
    chunk.emit(Op::Dup, loc());
    emit_constant(chunk, Value::Integer(0));
    emit_constant(chunk, Value::Integer(0));
    emit_constant(chunk, Value::Integer(80));
    emit_constant(chunk, Value::Integer(1));
    emit_constant(chunk, Value::Array(vec![menu_bar_file_item()]));
    emit_constant(chunk, default_menu_bar_style());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateMenuBarView,
        ))),
        loc(),
    );
    emit_constant(chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
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
fn tui_query_screen_line_reads_menu_bar_row() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_menu_bar(&mut chunk);
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
    assert![
        line.chars().nth(1) == Some('F'),
        "accel letter should be at column 2, got {line:?}"
    ];
}

#[test]
fn tui_query_screen_cell_reads_menu_bar_accel_color() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_menu_bar(&mut chunk);
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

    assert_eq!(worker.stack.last(), Some(&tui_screen_cell_value('F', 4, 7)));
}

//! VM tests for native TUI query intrinsics (Phase 3–4).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md`, `docs/future/tui-tests-fpas/README.md`

use crate::tests::helpers::{
    emit_constant, key_event_value, loc, minimal_shared_state, tui_application_value,
    tui_rect_value, tui_screen_cell_value, tui_size_value, tui_view_id_option_some,
    tui_view_id_value,
};
use crate::vm::Worker;
use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ConsoleKeyEvent;
use fpas_std::key_event::key_kind_index;
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
            ("Shortcut".into(), Value::Char('X')),
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
            ("Shortcut".into(), Value::Char('F')),
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
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(80));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Array(vec![menu_bar_file_item()]));
    emit_constant(&mut chunk, default_menu_bar_style());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateMenuBarView,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
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
    assert!(
        line.chars().nth(1) == Some('F'),
        "accel letter should be at column 2, got {line:?}"
    );
}

#[test]
fn tui_query_screen_cell_reads_menu_bar_accel_color() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(80));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Array(vec![menu_bar_file_item()]));
    emit_constant(&mut chunk, default_menu_bar_style());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateMenuBarView,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::TestPump))),
        loc(),
    );
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

#[test]
fn tui_query_root_views_lists_registered_roots_in_order() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(80));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Array(vec![menu_bar_file_item()]));
    emit_constant(&mut chunk, default_menu_bar_style());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateMenuBarView,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(80));
    emit_constant(&mut chunk, Value::Integer(24));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::OptionNone);
    emit_constant(&mut chunk, Value::OptionNone);
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateSolidFillView,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryRootViews))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query root views should succeed");

    let Value::Array(roots) = worker.stack.last().expect("roots on stack") else {
        panic!("expected root view array");
    };
    assert_eq!(roots, &[tui_view_id_value(0), tui_view_id_value(1),]);
}

fn emit_host_register_view(chunk: &mut Chunk, x: i64, y: i64, width: i64, height: i64) {
    emit_constant(chunk, tui_application_value());
    emit_constant(chunk, Value::Integer(x));
    emit_constant(chunk, Value::Integer(y));
    emit_constant(chunk, Value::Integer(width));
    emit_constant(chunk, Value::Integer(height));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
}

fn emit_host_set_view_parent(chunk: &mut Chunk, child: u32, parent: u32) {
    emit_constant(chunk, tui_application_value());
    emit_constant(chunk, tui_view_id_value(child));
    emit_constant(chunk, tui_view_id_option_some(parent));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewParent))),
        loc(),
    );
}

#[test]
fn tui_query_view_rect_returns_absolute_geometry() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_host_register_view(&mut chunk, 0, 0, 80, 1);
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, tui_view_id_value(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryViewRect))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query view rect should succeed");

    assert_eq!(worker.stack.last(), Some(&tui_rect_value(0, 0, 80, 1)));
}

#[test]
fn tui_query_view_parent_and_children_reflect_tree() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_host_register_view(&mut chunk, 0, 0, 40, 20);
    emit_host_register_view(&mut chunk, 1, 1, 10, 5);
    emit_host_register_view(&mut chunk, 12, 1, 10, 5);
    emit_host_set_view_parent(&mut chunk, 1, 0);
    emit_host_set_view_parent(&mut chunk, 2, 0);

    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, tui_view_id_value(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryViewParent))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, tui_view_id_value(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryViewChildren))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("view tree queries should succeed");

    let Value::Array(children) = worker.stack.last().expect("children on stack") else {
        panic!("expected children array");
    };
    assert_eq!(children, &[tui_view_id_value(1), tui_view_id_value(2)]);

    let parent = worker
        .stack
        .get(worker.stack.len() - 2)
        .expect("parent on stack");
    assert_eq!(parent, &Value::OptionSome(Box::new(tui_view_id_value(0))));
}

fn tui_menu_bar_state_value(
    menu_active: bool,
    hovered_index: i64,
    submenu_open: bool,
    submenu_bar_index: i64,
    selected_entry: i64,
) -> Value {
    Value::Record {
        type_name: "Std.Tui.MenuBarState".into(),
        fields: vec![
            ("menuActive".into(), Value::Boolean(menu_active)),
            ("hoveredIndex".into(), Value::Integer(hovered_index)),
            ("submenuOpen".into(), Value::Boolean(submenu_open)),
            ("submenuBarIndex".into(), Value::Integer(submenu_bar_index)),
            ("selectedEntry".into(), Value::Integer(selected_entry)),
        ],
    }
}

#[test]
fn tui_query_menu_bar_state_reflects_submenu_after_alt_shortcut() {
    let mut chunk = Chunk::new();
    emit_open_for_test(&mut chunk, 80, 25);
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(80));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Array(vec![menu_bar_file_item()]));
    emit_constant(&mut chunk, default_menu_bar_style());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateMenuBarView,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(
        &mut chunk,
        key_event_value(ConsoleKeyEvent::new(
            key_kind_index("Character"),
            'f',
            false,
            false,
            true,
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
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, tui_view_id_value(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryMenuBarState))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(shared);
    worker.run().expect("query menu bar state should succeed");

    assert_eq!(
        worker.stack.last(),
        Some(&tui_menu_bar_state_value(true, 0, true, 0, 0))
    );
}

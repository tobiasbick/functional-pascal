//! VM bridge tests for frame-root host calls and chrome dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use super::*;
use fpas_std::{
    ConsoleEvent, FrameCapabilities, FrameContentSize, FrameKind, FrameRootSpec,
    mouse_action_index, mouse_button_index,
};

#[test]
fn tui_host_create_frame_view_and_query_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(20));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Str("Window".into()));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(true));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostCreateFrameView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let root = tui.views.roots()[0];
    assert_eq!(
        tui.views.rect(root),
        Some(ViewRect {
            x: 10,
            y: 4,
            width: 20,
            height: 8,
        })
    );
    assert!(tui.views.frame_root_state(root).is_some());
}

#[test]
fn tui_host_frame_title_drag_updates_rect() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(20));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Str("Window".into()));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Boolean(true));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostCreateFrameView))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut key_input = shared.key_input.lock().unwrap_or_else(|e| e.into_inner());
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Down"),
            mouse_button_index("Left"),
            16,
            5,
            false,
            false,
            false,
            false,
        ));
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Move"),
            mouse_button_index("Left"),
            20,
            8,
            false,
            false,
            false,
            false,
        ));
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Up"),
            mouse_button_index("Left"),
            20,
            8,
            false,
            false,
            false,
            false,
        ));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let root = tui.views.roots()[0];
    assert_eq!(
        tui.views.rect(root),
        Some(ViewRect {
            x: 14,
            y: 7,
            width: 20,
            height: 8,
        })
    );
}

#[test]
fn tui_host_frame_title_drag_marks_descendant_subtree_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let frame = tui
            .views
            .register_frame_root(FrameRootSpec {
                kind: FrameKind::Window,
                outer: ViewRect {
                    x: 10,
                    y: 4,
                    width: 20,
                    height: 8,
                },
                content_size: FrameContentSize::new(0, 0),
                capabilities: FrameCapabilities {
                    movable: true,
                    resizable: false,
                    zoomable: false,
                    closable: false,
                    scrollable: false,
                },
                options: Default::default(),
            })
            .expect("frame root");
        let child = tui.views.register(ViewRect {
            x: 0,
            y: 0,
            width: 30,
            height: 2,
        });
        assert!(tui.views.set_parent(child, Some(frame.view_id)));

        let mut key_input = shared.key_input.lock().unwrap_or_else(|e| e.into_inner());
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Down"),
            mouse_button_index("Left"),
            16,
            5,
            false,
            false,
            false,
            false,
        ));
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Move"),
            mouse_button_index("Left"),
            20,
            8,
            false,
            false,
            false,
            false,
        ));
        key_input.push_console_event(ConsoleEvent::mouse(
            mouse_action_index("Up"),
            mouse_button_index("Left"),
            20,
            8,
            false,
            false,
            false,
            false,
        ));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let root = tui.views.roots()[0];
    assert_eq!(
        tui.views.rect(root),
        Some(ViewRect {
            x: 14,
            y: 7,
            width: 20,
            height: 8,
        })
    );

    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    let Some(DamageRegion::Rect(damage_rect)) = damage else {
        panic!("expected merged rect damage, got {damage:?}");
    };
    // Root-only before.union(after) would be width 24; the wide child extends past the frame outer.
    assert!(
        damage_rect.width > 24,
        "damage should include descendant cells outside the frame outer rect, got {damage_rect:?}"
    );
}

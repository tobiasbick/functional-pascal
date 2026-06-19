use super::*;

#[test]
fn tui_host_register_view_marks_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let damage = shared
        .tui
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 3,
            y: 4,
            width: 5,
            height: 6,
        }))
    );
}

#[test]
fn tui_host_register_view_rejects_non_positive_size() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("requires Width and Height greater than zero"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_push_child_view_rejects_unknown_view_id() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(99));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("Unknown host view handle 99"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_attach_view_to_active_modal_rejects_unknown_view_id() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(99));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostAttachViewToActiveModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("Unknown host view handle 99"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_unregister_view_marks_removed_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 8,
            y: 2,
            width: 7,
            height: 4,
        });
        assert_eq!(view_id.raw(), 0);
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let damage = shared
        .tui
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 8,
            y: 2,
            width: 7,
            height: 4,
        }))
    );
}

#[test]
fn tui_host_unregister_focused_view_marks_removed_and_new_focus_rects() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let first = tui.views.register(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: 3,
        });
        let second = tui.views.register(ViewRect {
            x: 10,
            y: 2,
            width: 5,
            height: 4,
        });
        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        tui.views.push_child(first);
        tui.views.push_child(second);
        let _ = tui.views.focus_next();
        let _ = tui.views.focus_next();
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 1,
            y: 1,
            width: 14,
            height: 5,
        }))
    );
    assert_eq!(tui.views.focused_id().map(|id| id.raw()), Some(0));
}

#[test]
fn tui_host_set_view_rect_marks_old_and_new_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(0));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 1,
            y: 2,
            width: 5,
            height: 4,
        });
        assert_eq!(view_id.raw(), 0);
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 1,
            y: 2,
            width: 9,
            height: 8,
        }))
    );
    assert_eq!(
        tui.views.rect(fpas_std::ViewId::from_raw(0)),
        Some(ViewRect {
            x: 3,
            y: 4,
            width: 7,
            height: 6,
        })
    );
}

#[test]
fn tui_host_set_child_view_rect_marks_resolved_screen_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(1));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let parent = tui.views.register(ViewRect {
            x: 10,
            y: 20,
            width: 30,
            height: 20,
        });
        let child = tui.views.register(ViewRect {
            x: 11,
            y: 22,
            width: 5,
            height: 4,
        });
        assert!(tui.views.set_parent(child, Some(parent)));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 11,
            y: 22,
            width: 9,
            height: 8,
        }))
    );
    assert_eq!(
        tui.views.rect(ViewId::from_raw(1)),
        Some(ViewRect {
            x: 13,
            y: 24,
            width: 7,
            height: 6,
        })
    );
}

#[test]
fn tui_host_set_view_rect_ignores_unknown_view_ids() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(99));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(damage, None);
}

#[test]
fn tui_host_set_view_rect_rejects_non_positive_size_without_mutation() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, tui_view_id_value(0));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(-1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let original = ViewRect {
        x: 1,
        y: 2,
        width: 5,
        height: 4,
    };
    shared
        .tui
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .views
        .register(original);

    let mut worker = Worker::new_main(Arc::clone(&shared));
    let error = worker.run().expect_err("invalid geometry should fail");
    assert!(
        error
            .message
            .contains("requires Width and Height greater than zero"),
        "unexpected runtime error: {}",
        error.message
    );

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.views.rect(ViewId::from_raw(0)), Some(original));
    assert_eq!(
        tui.session
            .peek_redraw_damage(loc())
            .expect("peek damage should succeed"),
        None
    );
}

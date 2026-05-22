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
fn tui_host_unregister_view_marks_removed_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
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
    emit_constant(&mut chunk, Value::Integer(1));
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
    emit_constant(&mut chunk, Value::Integer(0));
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
fn tui_host_set_view_rect_ignores_unknown_view_ids() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(99));
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

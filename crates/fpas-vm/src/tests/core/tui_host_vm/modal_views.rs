use super::*;
use fpas_std::ModalResult;

#[test]
fn tui_host_modal_depth_tracks_enter_and_leave() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryModalDepth))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::QueryModalDepth))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2", "1"]);
}

#[test]
fn tui_host_set_active_modal_result_accepts_builtin_and_command_codes() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(1007));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.active_result(), Some(ModalResult::Command(1007)));
}

#[test]
fn tui_host_set_active_modal_result_rejects_invalid_code_without_mutation() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(999));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    let error = worker.run().expect_err("invalid modal result should fail");
    assert!(
        error.message.contains(
            "expects 1 (Accept), 2 (Cancel), or an application-defined result code >= 1000"
        ),
        "unexpected runtime error: {}",
        error.message
    );

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.active_result(), Some(ModalResult::Accept));
}

#[test]
fn tui_host_set_active_modal_result_requires_active_modal() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostSetActiveModalResult,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("requires an active modal frame"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_enter_modal_without_attached_views_does_not_mark_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
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
    assert_eq!(damage, None);
}

#[test]
fn tui_host_attach_view_to_active_modal_marks_view_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostAttachViewToActiveModal,
        ))),
        loc(),
    );
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
            x: 10,
            y: 11,
            width: 6,
            height: 7,
        }))
    );
}

#[test]
fn tui_host_leave_modal_marks_popped_modal_view_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostAttachViewToActiveModal,
        ))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))),
        loc(),
    );
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
            x: 10,
            y: 11,
            width: 6,
            height: 7,
        }))
    );
}

#[test]
fn tui_host_leave_modal_marks_popped_and_revealed_modal_view_rects() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(3));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostAttachViewToActiveModal,
        ))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostAttachViewToActiveModal,
        ))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))),
        loc(),
    );
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
            x: 1,
            y: 1,
            width: 14,
            height: 5,
        }))
    );
}

#[test]
fn tui_host_modal_stack_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
}

#[test]
fn tui_application_show_dialog_registers_owned_root_and_close_modal_removes_it() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(8));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowDialog,
        ))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationCloseModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
    assert!(tui.views.is_empty());
}

#[test]
fn tui_application_show_framed_dialog_geometry_error_is_atomic() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Str("Too small".into()));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    emit_constant(&mut chunk, Value::Boolean(false));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowFramedDialog,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    let error = worker.run().expect_err("invalid frame geometry must fail");
    assert!(error.message.contains("requires at least 4x3"));

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
    assert!(tui.views.is_empty());
    assert!(tui.view_widgets.is_empty());
}

#[test]
fn tui_application_show_modal_restores_previous_focus_on_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, tui_view_id_value(2));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowModal,
        ))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationCloseModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let root = tui.views.register(ViewRect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        });
        let leaf = tui.views.register(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: 1,
        });
        let modal_root = tui.views.register(ViewRect {
            x: 5,
            y: 3,
            width: 20,
            height: 10,
        });
        let modal_leaf = tui.views.register(ViewRect {
            x: 6,
            y: 4,
            width: 4,
            height: 1,
        });
        assert_eq!(root.raw(), 0);
        assert_eq!(leaf.raw(), 1);
        assert_eq!(modal_root.raw(), 2);
        assert_eq!(modal_leaf.raw(), 3);
        assert!(tui.views.set_parent(leaf, Some(root)));
        assert!(tui.views.set_parent(modal_leaf, Some(modal_root)));
        assert!(tui.views.push_child(leaf));
        assert!(tui.views.push_child(modal_leaf));
        assert_eq!(tui.views.focus_view(leaf), (true, false));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
    assert_eq!(tui.views.focused_id(), Some(ViewId::from_raw(1)));
    assert_eq!(tui.views.active_root(), Some(ViewId::from_raw(0)));
    assert_eq!(
        tui.views.rect(ViewId::from_raw(2)).map(|rect| rect.width),
        Some(20)
    );
}

#[test]
fn tui_application_close_inner_modal_restores_outer_modal_focus() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, tui_view_id_value(2));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(20));
    emit_constant(&mut chunk, tui_view_id_value(4));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowModal,
        ))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationCloseModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let root = tui.views.register(ViewRect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        });
        let leaf = tui.views.register(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: 1,
        });
        let outer_root = tui.views.register(ViewRect {
            x: 4,
            y: 2,
            width: 20,
            height: 10,
        });
        let outer_leaf = tui.views.register(ViewRect {
            x: 5,
            y: 3,
            width: 4,
            height: 1,
        });
        let inner_root = tui.views.register(ViewRect {
            x: 8,
            y: 4,
            width: 20,
            height: 10,
        });
        let inner_leaf = tui.views.register(ViewRect {
            x: 9,
            y: 5,
            width: 4,
            height: 1,
        });
        assert_eq!(root.raw(), 0);
        assert_eq!(leaf.raw(), 1);
        assert_eq!(outer_root.raw(), 2);
        assert_eq!(outer_leaf.raw(), 3);
        assert_eq!(inner_root.raw(), 4);
        assert_eq!(inner_leaf.raw(), 5);
        assert!(tui.views.set_parent(leaf, Some(root)));
        assert!(tui.views.set_parent(outer_leaf, Some(outer_root)));
        assert!(tui.views.set_parent(inner_leaf, Some(inner_root)));
        assert!(tui.views.push_child(leaf));
        assert!(tui.views.push_child(outer_leaf));
        assert!(tui.views.push_child(inner_leaf));
        assert_eq!(tui.views.focus_view(leaf), (true, false));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 1);
    assert_eq!(tui.modals.active_root_view(), Some(ViewId::from_raw(2)));
    assert_eq!(tui.views.focused_id(), Some(ViewId::from_raw(3)));
    assert_eq!(tui.views.active_root(), Some(ViewId::from_raw(2)));
}

#[test]
fn tui_application_show_dialog_close_restores_previous_focus_after_owned_root_removal() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(8));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationShowDialog,
        ))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationCloseModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let root = tui.views.register(ViewRect {
            x: 0,
            y: 0,
            width: 20,
            height: 10,
        });
        let leaf = tui.views.register(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: 1,
        });
        assert_eq!(root.raw(), 0);
        assert_eq!(leaf.raw(), 1);
        assert!(tui.views.set_parent(leaf, Some(root)));
        assert!(tui.views.push_child(leaf));
        assert_eq!(tui.views.focus_view(leaf), (true, false));
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
    assert_eq!(tui.views.len(), 2);
    assert_eq!(tui.views.focused_id(), Some(ViewId::from_raw(1)));
    assert_eq!(tui.views.active_root(), Some(ViewId::from_raw(0)));
}

use super::*;

#[test]
fn host_register_view_returns_distinct_integer_handles() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["0", "1"]);
}

#[test]
fn host_push_child_view_populates_focus_chain_and_query_focused_view_id() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostQueryFocusedViewId,
        ))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["14", "Some(0)"]);
}

#[test]
fn host_unregister_view_removes_it_from_focus_chain() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView))),
        loc(),
    );

    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostQueryFocusedViewId,
        ))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["None"]);

    let tui = shared.tui.lock().unwrap();
    assert!(!tui.views.has_focusable_children());
}

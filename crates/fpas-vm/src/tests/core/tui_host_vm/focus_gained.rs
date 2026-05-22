use super::*;

#[test]
fn tui_host_register_on_focus_gained_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
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
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_focus_gained.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnFocusGained"
        ),
        "on_focus_gained should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_focus_gained_handler_returns_tag_ten() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
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
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Str("fg".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_gained());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["fg", "10"]);
}

#[test]
fn tui_host_focus_gained_redraw_hint_uses_focused_view_rect_when_handler_requests_redraw() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGainedRedraw".into(),
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
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGainedRedraw".into(), (on_fg_start, 2));
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationRequestRedraw,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 9,
            y: 2,
            width: 5,
            height: 4,
        });
        tui.views.push_child(view_id);
        let _ = tui.views.focus_next();
    }
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::focus_gained());

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
            x: 9,
            y: 2,
            width: 5,
            height: 4,
        }))
    );
}

#[test]
fn tui_host_process_next_focus_gained_without_handler_returns_tag_eleven() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_gained());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["11"]);
}

#[test]
fn tui_host_register_on_focus_gained_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
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
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGained".into(), (on_fg_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_focus_gained.is_none(),
        "on_focus_gained should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_focus_gained_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFG".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusGained,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk.functions.insert("WrongFG".into(), (on_fg_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("OnFocusGained handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

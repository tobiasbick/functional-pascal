use super::*;

#[test]
fn tui_host_register_on_paste_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_paste.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnPaste"
        ),
        "on_paste should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_paste_handler_returns_tag_eight() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
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

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Str("paste".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::paste("hello".to_string()));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["paste", "8"]);
}

#[test]
fn tui_host_paste_redraw_hint_uses_focused_view_rect_when_handler_requests_redraw() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPasteRedraw".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
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

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPasteRedraw".into(), (on_paste_start, 2));
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
            x: 2,
            y: 1,
            width: 6,
            height: 3,
        });
        tui.views.push_child(view_id);
        let _ = tui.views.focus_next();
    }
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::paste("hello".to_string()));

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
            x: 2,
            y: 1,
            width: 6,
            height: 3,
        }))
    );
}

#[test]
fn tui_host_process_next_paste_without_handler_returns_tag_nine() {
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
    vm.push_console_event(ConsoleEvent::paste("world".to_string()));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["9"]);
}

#[test]
fn tui_host_register_on_paste_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPaste".into(), (on_paste_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_paste.is_none(),
        "on_paste should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_paste_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnPaste expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_paste_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnPaste".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnPaste".into(), (on_paste_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnPaste handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

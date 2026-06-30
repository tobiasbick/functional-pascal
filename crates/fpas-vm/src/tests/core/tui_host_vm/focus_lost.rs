use super::*;

#[test]
fn tui_host_register_on_focus_lost_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusLost,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk.insert_function("OnFocusLost", on_fl_start, 2);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_focus_lost.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnFocusLost"
        ),
        "on_focus_lost should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_focus_lost_handler_returns_tag_twelve() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusLost,
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

    let on_fl_start = chunk.len();
    chunk.insert_function("OnFocusLost", on_fl_start, 2);
    emit_constant(&mut chunk, Value::Str("fl".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::focus_lost());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["fl", "12"]);
}

#[test]
fn tui_host_process_next_focus_lost_without_handler_returns_tag_thirteen() {
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
    vm.push_console_event(ConsoleEvent::focus_lost());
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["13"]);
}

#[test]
fn tui_host_register_on_focus_lost_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusLost,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk.insert_function("OnFocusLost", on_fl_start, 2);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_focus_lost.is_none(),
        "on_focus_lost should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_focus_lost_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFL".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnFocusLost,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk.insert_function("WrongFL", on_fl_start, 1);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("OnFocusLost handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

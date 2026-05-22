use super::*;

#[test]
fn tui_host_run_loop_dispatches_paint_then_key_until_idle() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationRequestRedraw,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnKeyPressed,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(16));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.functions.insert("OnKey".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["p"]);
}

#[test]
fn tui_host_request_quit_ends_host_run_loop_after_idle_iteration() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(10_000));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("vm ok");
    assert!(
        vm.output().lines.is_empty(),
        "run loop should exit on quit after first idle iteration without printing"
    );
}

#[test]
fn tui_host_run_loop_max_iterations_zero_skips_body() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::ApplicationRequestRedraw,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert!(run_ok_output(chunk).is_empty());
}

#[test]
fn tui_host_register_on_exit_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(matches!(
        tui.on_exit.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnExit"
    ));
}

#[test]
fn tui_host_register_on_idle_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_idle.is_none(),
        "Application.Close should clear OnIdle"
    );
    assert_eq!(tui.idle_interval_ms, 0);
}

#[test]
fn tui_host_register_on_idle_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnIdle expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_idle_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnIdle".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnIdle".into(), (on_idle_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnIdle handler must have arity 1"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_exit_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk.functions.insert("OnExit".into(), (on_exit_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_exit.is_none(),
        "Application.Close should clear OnExit"
    );
}

#[test]
fn tui_host_register_on_exit_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnExit expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_exit_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnExit".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_exit_start = chunk.len();
    chunk
        .functions
        .insert("WrongOnExit".into(), (on_exit_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnExit handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

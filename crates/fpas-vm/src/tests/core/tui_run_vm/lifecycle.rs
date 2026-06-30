use super::*;

#[test]
fn tui_application_run_invokes_on_exit_and_clears_shared_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
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
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk.insert_function("OnPaint", on_paint_start, 1);
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.insert_function("OnExit", on_exit_start, 2);
    emit_constant(&mut chunk, Value::Str("x".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    assert_eq!(
        worker
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .lines
            .clone(),
        vec!["p", "x"],
    );

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_paint.is_none(),
        "Application.Run should clear OnPaint"
    );
    assert!(tui.on_exit.is_none(), "Application.Run should clear OnExit");
    assert!(
        tui.last_exit_reason.is_none(),
        "Application.Run close semantics should clear the last exit reason"
    );
    assert!(
        !tui.run_active,
        "Application.Run should reset the active-run guard"
    );
}

#[test]
fn tui_application_run_reports_host_stop_when_close_happens_during_run() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk.insert_function("OnPaint", on_paint_start, 1);
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.insert_function("OnExit", on_exit_start, 2);
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    assert_eq!(
        worker
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .lines
            .clone(),
        vec!["Std.Tui.ExitReason.HostStop"],
    );

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_paint.is_none(),
        "Application.Run should clear OnPaint after HostStop"
    );
    assert!(
        tui.on_exit.is_none(),
        "Application.Run should clear OnExit after HostStop"
    );
    assert!(
        tui.last_exit_reason.is_none(),
        "Application.Run close semantics should clear the last exit reason after HostStop"
    );
    assert!(
        !tui.host_stop_requested,
        "Application.Run should clear the pending HostStop request"
    );
    assert!(
        !tui.run_active,
        "Application.Run should reset the active-run guard after HostStop"
    );
}

#[test]
fn tui_application_run_reports_host_and_user_stop_when_both_are_requested() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk.insert_function("OnPaint", on_paint_start, 1);
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.insert_function("OnExit", on_exit_start, 2);
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    assert_eq!(
        worker
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .lines
            .clone(),
        vec!["Std.Tui.ExitReason.HostAndUserStop"],
    );
}

#[test]
fn tui_application_run_reports_host_shutdown_when_vm_shutdown_is_requested() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk.insert_function("OnPaint", on_paint_start, 1);
    emit_constant(&mut chunk, Value::Str("paint".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_exit_start = chunk.len();
    chunk.insert_function("OnExit", on_exit_start, 2);
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let shutdown_shared = Arc::clone(&shared);
    let shutdown_thread = thread::spawn(move || {
        loop {
            let painted = {
                let console = shutdown_shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                console.output().lines.iter().any(|line| line == "paint")
            };
            if painted {
                break;
            }
            thread::yield_now();
        }
        shutdown_shared.request_shutdown();
    });

    let mut worker = Worker::new_main(Arc::clone(&shared));
    let error = worker
        .run()
        .expect_err("VM should fail due to requested shutdown");
    shutdown_thread
        .join()
        .expect("shutdown helper thread should join cleanly");

    assert!(
        error
            .message
            .contains("Execution aborted: a concurrent task failed"),
        "unexpected runtime error: {}",
        error.message
    );
    assert_eq!(
        worker
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output()
            .lines
            .clone(),
        vec!["paint", "Std.Tui.ExitReason.HostShutdown"],
    );
}

#[test]
fn tui_application_run_rejects_missing_on_paint_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error
            .message
            .contains("Application.Run(App) requires a registered OnPaint handler"),
        "unexpected runtime error: {}",
        error.message
    );
}

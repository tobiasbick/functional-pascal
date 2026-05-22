use super::*;

#[test]
fn tui_application_run_dispatches_initial_resize_before_first_paint() {
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
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnResize,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(1));
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRun))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("paint".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_resize_start = chunk.len();
    chunk
        .functions
        .insert("OnResize".into(), (on_resize_start, 2));
    emit_constant(&mut chunk, Value::Str("resize".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Str("idle".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut key_input = shared.key_input.lock().unwrap_or_else(|e| e.into_inner());
        key_input.push_console_event(ConsoleEvent::resize(120, 40));
        key_input.push_console_event(ConsoleEvent::resize(121, 41));
    }

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
        vec!["resize", "paint", "idle"],
    );
}

use super::*;

#[test]
fn tui_host_dispatch_redraw_invokes_on_paint() {
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
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["p", "5"]);
}

#[test]
fn tui_host_dispatch_redraw_consumes_damage_only_once() {
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
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Str("p".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["p", "5", "0"]);
}

#[test]
fn tui_host_dispatch_redraw_without_handler_clears_and_returns_six() {
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
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["6"]);
}

#[test]
fn tui_host_dispatch_redraw_when_not_pending_returns_zero() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["0"]);
}

#[test]
fn tui_host_dispatch_redraw_runs_handler_attached_to_widget_view() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::OptionNone);
    emit_constant(&mut chunk, Value::OptionNone);
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostCreateSolidFillView,
        ))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnViewPaint".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnViewPaint,
        ))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_view_paint_start = chunk.len();
    chunk
        .functions
        .insert("OnViewPaint".into(), (on_view_paint_start, 3));
    emit_constant(&mut chunk, Value::Str("view".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["view", "5"]);
}

#[test]
fn tui_host_global_on_paint_clips_partial_damage() {
    let chunk = build_function_chunk(
        "OnPaint",
        1,
        |main| {
            emit_constant(main, Value::Integer(20));
            emit_constant(main, Value::Integer(10));
            main.emit(
                Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::OpenForTest))),
                loc(),
            );
            main.emit(Op::Dup, loc());
            emit_constant(main, Value::Integer(0));
            emit_constant(main, Value::Integer(0));
            emit_constant(main, Value::Integer(20));
            emit_constant(main, Value::Integer(10));
            emit_constant(main, Value::Integer(1));
            emit_constant(main, Value::OptionNone);
            emit_constant(main, Value::OptionSome(Box::new(Value::Str("M".into()))));
            main.emit(
                Op::Intrinsic(u16::from(Intrinsic::Tui(
                    TuiIntrinsic::HostCreateSolidFillView,
                ))),
                loc(),
            );
            main.emit(Op::Pop, loc());
            main.emit(Op::Dup, loc());
            main.emit(
                Op::Intrinsic(u16::from(Intrinsic::Tui(
                    TuiIntrinsic::ApplicationRequestRedraw,
                ))),
                loc(),
            );
            main.emit(
                Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw))),
                loc(),
            );
            main.emit(Op::Pop, loc());
            emit_constant(main, tui_application_value());
            emit_constant(
                main,
                Value::Function {
                    name: "OnPaint".into(),
                    captures: vec![],
                },
            );
            main.emit(
                Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint))),
                loc(),
            );
        },
        |body| {
            body.emit(
                Op::Intrinsic(u16::from(Intrinsic::Console(
                    fpas_bytecode::ConsoleIntrinsic::ClrScr,
                ))),
                loc(),
            );
            emit_constant(body, Value::Unit);
            body.emit(Op::Return, loc());
        },
    );

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("setup should succeed");

    let marker = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .query_screen_cell(15, 5);
    assert_eq!(marker.map(|(ch, _, _)| ch), Some('M'));

    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        tui.view_widgets.clear();
        tui.views.clear();
        tui.session
            .request_redraw_rect(
                ViewRect {
                    x: 1,
                    y: 1,
                    width: 4,
                    height: 3,
                },
                loc(),
            )
            .expect("partial damage should be accepted");
    }

    assert_eq!(
        worker
            .tui_host_dispatch_redraw_inner(loc())
            .expect("redraw should succeed"),
        5
    );

    let console = shared.console.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        console.query_screen_cell(15, 5).map(|(ch, _, _)| ch),
        Some('M')
    );
    assert_eq!(
        console.query_screen_cell(2, 2).map(|(ch, _, _)| ch),
        Some(' ')
    );
}

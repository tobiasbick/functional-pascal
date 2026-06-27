use super::*;

#[test]
fn tui_host_register_on_mouse_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk.insert_function("OnMouse", on_mouse_start, 2);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        matches!(
            tui.on_mouse.as_ref(),
            Some(Value::Function { name, .. }) if name == "OnMouse"
        ),
        "on_mouse should store the registered handler"
    );
}

#[test]
fn tui_host_process_next_dispatches_on_mouse_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
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

    let on_mouse_start = chunk.len();
    chunk.insert_function("OnMouse", on_mouse_start, 2);
    emit_constant(&mut chunk, Value::Str("m".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    use fpas_std::{mouse_action_index, mouse_button_index};
    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::mouse(
        mouse_action_index("ScrollUp"),
        mouse_button_index("None"),
        5,
        3,
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["m", "5"]);
}

#[test]
fn tui_host_mouse_redraw_hint_uses_view_rect_when_handler_requests_redraw() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouseRedraw".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
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

    let on_mouse_start = chunk.len();
    chunk.insert_function("OnMouseRedraw", on_mouse_start, 2);
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
        tui.views.register(ViewRect {
            x: 5,
            y: 3,
            width: 4,
            height: 2,
        });
    }
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::mouse(
            fpas_std::mouse_action_index("Move"),
            fpas_std::mouse_button_index("None"),
            6,
            4,
            false,
            false,
            false,
            false,
        ));

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
            x: 5,
            y: 3,
            width: 4,
            height: 2,
        }))
    );
}

#[test]
fn tui_host_process_next_mouse_without_handler_returns_tag_seven() {
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

    use fpas_std::{mouse_action_index, mouse_button_index};
    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::mouse(
        mouse_action_index("Down"),
        mouse_button_index("Left"),
        1,
        1,
        false,
        false,
        false,
        false,
    ));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["7"]);
}

#[test]
fn tui_host_register_on_mouse_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk.insert_function("OnMouse", on_mouse_start, 2);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        tui.on_mouse.is_none(),
        "on_mouse should be cleared on close"
    );
}

#[test]
fn tui_host_register_on_mouse_rejects_non_function_value() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnMouse expects a function value"),
        "unexpected runtime error: {}",
        error.message
    );
}

#[test]
fn tui_host_register_on_mouse_rejects_wrong_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongOnMouse".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk.insert_function("WrongOnMouse", on_mouse_start, 1);
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnMouse handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

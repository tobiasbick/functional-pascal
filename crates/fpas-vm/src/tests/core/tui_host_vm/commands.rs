use super::*;

#[test]
fn tui_host_invoke_on_key_pressed_runs_registered_fp_function() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
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
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(
        &mut chunk,
        key_event_value(ConsoleKeyEvent::new(
            key_kind_index("Space"),
            ' ',
            false,
            false,
            false,
            false,
        )),
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostInvokeOnKeyPressed,
        ))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_key_start = chunk.len();
    chunk.functions.insert("OnKey".into(), (on_key_start, 2));
    emit_constant(&mut chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["true"]);
}

#[test]
fn tui_host_command_shortcut_dispatches_on_command_and_returns_tag_sixteen() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnCommand,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_command_start = chunk.len();
    chunk
        .functions
        .insert("OnCommand".into(), (on_command_start, 2));
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::key(save_key));
    vm.run().expect("vm ok");

    assert_eq!(vm.output().lines, vec!["42", "16"]);
}

#[test]
fn tui_host_bound_command_without_handler_returns_tag_seventeen() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::key(save_key));
    vm.run().expect("vm ok");

    assert_eq!(vm.output().lines, vec!["17"]);
}

#[test]
fn tui_host_bind_command_to_view_stores_local_binding() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostBindCommandToView,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let binding = tui
        .view_commands
        .get(&ViewId::from_raw(0))
        .and_then(|commands| commands.resolve(&save_key));
    assert_eq!(binding, Some(fpas_std::CommandId(20)));
}

#[test]
fn tui_host_bind_command_to_active_modal_stores_modal_binding() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

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
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(30));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostBindCommandToActiveModal,
        ))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        tui.modals.resolve_active_command(&save_key),
        Some(fpas_std::CommandId(30))
    );
}

#[test]
fn tui_host_view_command_shortcut_uses_focused_ancestor_binding() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnCommand,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::MakeSome, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewParent))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostBindCommandToView,
        ))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_command_start = chunk.len();
    chunk
        .functions
        .insert("OnCommand".into(), (on_command_start, 2));
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Tab"),
        '\t',
        false,
        false,
        false,
        false,
    )));
    vm.push_console_event(ConsoleEvent::key(save_key));
    vm.run().expect("vm ok");

    assert_eq!(vm.output().lines, vec!["20", "16"]);
}

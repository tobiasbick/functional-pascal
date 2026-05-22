use super::*;

#[test]
fn tab_with_two_focusable_views_fires_on_activate_and_returns_tag_14() {
    let mut chunk =
        build_process_next_chunk_with_handlers(Some("OnActivate"), Some("OnDeactivate"), None);
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_handler(&mut chunk, "OnDeactivate", 1, "deactivate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["activate", "14"]);
}

#[test]
fn tab_second_press_fires_deactivate_then_activate() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnActivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnActivate,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnDeactivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnDeactivate,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_handler(&mut chunk, "OnDeactivate", 1, "deactivate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["activate", "deactivate", "activate"]);
}

#[test]
fn first_focus_transition_marks_rect_damage_for_focused_view() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, None);
    add_handler(&mut chunk, "OnActivate", 1, "activate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let focused = tui.views.register(view_rect_at(4, 2, 7, 3));
        tui.views.push_child(focused);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let damage = shared
        .tui
        .lock()
        .unwrap()
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(damage, Some(DamageRegion::Rect(view_rect_at(4, 2, 7, 3))));
}

#[test]
fn second_focus_transition_merges_previous_and_current_rects() {
    let mut chunk = Chunk::new();
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let first = tui.views.register(view_rect_at(1, 1, 4, 3));
        let second = tui.views.register(view_rect_at(10, 2, 5, 4));
        tui.views.push_child(first);
        tui.views.push_child(second);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let damage = shared
        .tui
        .lock()
        .unwrap()
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(damage, Some(DamageRegion::Rect(view_rect_at(1, 1, 14, 5))));
}

#[test]
fn shift_tab_fires_on_activate_and_returns_tag_15() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, None);
    add_handler(&mut chunk, "OnActivate", 1, "activate");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        let b = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.push_child(b);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(true));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["activate", "15"]);
}

#[test]
fn tab_with_no_focusable_views_dispatches_to_on_key_pressed() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["key", "1"]);
}

#[test]
fn tab_with_single_unfocused_view_establishes_focus_fires_on_activate() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        tui.views.push_child(a);
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["activate", "14"]);
}

#[test]
fn tab_with_single_already_focused_view_falls_through_to_on_key_pressed() {
    let mut chunk = build_process_next_chunk_with_handlers(Some("OnActivate"), None, Some("OnKey"));
    add_handler(&mut chunk, "OnActivate", 1, "activate");
    add_key_handler(&mut chunk, "OnKey", "key");

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap();
        let a = tui.views.register(view_rect());
        tui.views.push_child(a);
        tui.views.focus_next();
    }

    shared
        .key_input
        .lock()
        .unwrap()
        .push_console_event(tab_event(false));

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let lines = shared.console.lock().unwrap().output().lines.clone();
    assert_eq!(lines, vec!["key", "1"]);
}

#[test]
fn close_after_run_clears_on_activate_and_on_deactivate() {
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
            name: "OnActivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnActivate,
        ))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnDeactivate".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(
            TuiIntrinsic::HostRegisterOnDeactivate,
        ))),
        loc(),
    );
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
    chunk
        .functions
        .insert("OnPaint".into(), (on_paint_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_activate_start = chunk.len();
    chunk
        .functions
        .insert("OnActivate".into(), (on_activate_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let on_deactivate_start = chunk.len();
    chunk
        .functions
        .insert("OnDeactivate".into(), (on_deactivate_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap();
    assert!(
        tui.on_activate.is_none(),
        "Application.Run should clear on_activate"
    );
    assert!(
        tui.on_deactivate.is_none(),
        "Application.Run should clear on_deactivate"
    );
}

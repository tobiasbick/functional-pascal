//! VM bridge for `TuiHost` (Phase 3): host poll/register/process intrinsics (see `docs/pascal/std/tui-app.md`).
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ConsoleEvent;
use fpas_std::ConsoleKeyEvent;
use fpas_std::DamageRegion;
use fpas_std::ViewId;
use fpas_std::ViewRect;
use fpas_std::key_event::key_kind_index;
use std::sync::Arc;

use crate::Vm;
use crate::tests::helpers::{
    emit_constant, key_event_value, loc, minimal_shared_state, run_err, run_ok_output,
    tui_application_value,
};
use crate::vm::Worker;

#[test]
fn tui_host_invoke_on_key_pressed_runs_registered_fp_function() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnKey".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed))),
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed))),
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
fn tui_host_register_view_marks_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
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
            x: 3,
            y: 4,
            width: 5,
            height: 6,
        }))
    );
}

#[test]
fn tui_host_unregister_view_marks_removed_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 8,
            y: 2,
            width: 7,
            height: 4,
        });
        assert_eq!(view_id.raw(), 0);
    }

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
            x: 8,
            y: 2,
            width: 7,
            height: 4,
        }))
    );
}

#[test]
fn tui_host_unregister_focused_view_marks_removed_and_new_focus_rects() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let first = tui.views.register(ViewRect {
            x: 1,
            y: 1,
            width: 4,
            height: 3,
        });
        let second = tui.views.register(ViewRect {
            x: 10,
            y: 2,
            width: 5,
            height: 4,
        });
        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        tui.views.push_child(first);
        tui.views.push_child(second);
        let _ = tui.views.focus_next();
        let _ = tui.views.focus_next();
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 1,
            y: 1,
            width: 14,
            height: 5,
        }))
    );
    assert_eq!(tui.views.focused_id().map(|id| id.raw()), Some(0));
}

#[test]
fn tui_host_set_view_rect_marks_old_and_new_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 1,
            y: 2,
            width: 5,
            height: 4,
        });
        assert_eq!(view_id.raw(), 0);
    }

    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(
        damage,
        Some(DamageRegion::Rect(ViewRect {
            x: 1,
            y: 2,
            width: 9,
            height: 8,
        }))
    );
    assert_eq!(
        tui.views.rect(fpas_std::ViewId::from_raw(0)),
        Some(ViewRect {
            x: 3,
            y: 4,
            width: 7,
            height: 6,
        })
    );
}

#[test]
fn tui_host_set_view_rect_ignores_unknown_view_ids() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(99));
    emit_constant(&mut chunk, Value::Integer(3));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(6));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    let damage = tui
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(damage, None);
}

#[test]
fn tui_host_command_shortcut_dispatches_on_command_and_returns_tag_sixteen() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(30));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToActiveModal))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostSetViewParent))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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

#[test]
fn tui_host_modal_command_shortcut_uses_active_modal_binding() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(8));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(30));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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

    assert_eq!(vm.output().lines, vec!["30", "16"]);
}

#[test]
fn tui_host_modal_depth_tracks_enter_and_leave() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostModalDepth))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostModalDepth))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    assert_eq!(run_ok_output(chunk), vec!["2", "1"]);
}

#[test]
fn tui_host_enter_modal_without_attached_views_does_not_mark_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let damage = shared
        .tui
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .peek_redraw_damage(loc())
        .expect("peek damage should succeed");
    assert_eq!(damage, None);
}

#[test]
fn tui_host_attach_view_to_active_modal_marks_view_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
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
            x: 10,
            y: 11,
            width: 6,
            height: 7,
        }))
    );
}

#[test]
fn tui_host_leave_modal_marks_popped_modal_view_rect_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(11));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRedrawPending))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
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
            x: 10,
            y: 11,
            width: 6,
            height: 7,
        }))
    );
}

#[test]
fn tui_host_leave_modal_marks_popped_and_revealed_modal_view_rects() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(4));
    emit_constant(&mut chunk, Value::Integer(3));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(2));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(4));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRedrawPending))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
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
            x: 1,
            y: 1,
            width: 14,
            height: 5,
        }))
    );
}

#[test]
fn tui_host_modal_stack_is_cleared_by_application_close() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
}

#[test]
fn tui_application_show_dialog_registers_owned_root_and_close_modal_removes_it() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(6));
    emit_constant(&mut chunk, Value::Integer(7));
    emit_constant(&mut chunk, Value::Integer(8));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationShowDialog))),
        loc(),
    );
    chunk.emit(Op::Pop, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationCloseModal))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.modals.depth(), 0);
    assert!(tui.views.is_empty());
}

#[test]
fn tui_host_modal_scope_blocks_command_when_focus_is_outside_scope() {
    let save_key =
        ConsoleKeyEvent::new(key_kind_index("Character"), 's', false, true, false, false);

    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnCommand".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, key_event_value(save_key.clone()));
    emit_constant(&mut chunk, Value::Integer(42));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostBindCommand))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(0));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(10));
    emit_constant(&mut chunk, Value::Integer(5));
    emit_constant(&mut chunk, Value::Integer(5));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(1), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPushChildView))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostEnterModal))), loc());
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(Op::GetLocal(2), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal))),
        loc(),
    );
    chunk.emit(Op::GetLocal(0), loc());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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

    assert_eq!(vm.output().lines, vec!["20"]);
}

#[test]
fn tui_host_poll_next_coalesces_resize_before_key() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPollNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPollNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostPollNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(10, 10));
    vm.push_console_event(ConsoleEvent::resize(30, 20));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    let lines = vm.output().lines;
    assert_eq!(lines[0], "None", "resize-only poll buffers coalesced size");
    assert_eq!(
        lines[1], "None",
        "second resize still waits for a key before flush"
    );
    assert!(
        lines[2].contains("30"),
        "third poll (key) flushes coalesced resize (width 30): {}",
        lines[2]
    );
}

#[test]
fn tui_host_process_next_dispatches_on_resize_handler() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_resize_start = chunk.len();
    chunk
        .functions
        .insert("OnResize".into(), (on_resize_start, 2));
    emit_constant(&mut chunk, Value::Str("r".into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(10, 10));
    vm.push_console_event(ConsoleEvent::resize(30, 20));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\u{1b}',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["r", "2"]);
}

#[test]
fn tui_host_process_next_resize_marks_union_of_surface_bounds() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnResize".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_resize_start = chunk.len();
    chunk
        .functions
        .insert("OnResize".into(), (on_resize_start, 2));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::resize(80, 25));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )));

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
            x: 0,
            y: 0,
            width: 80,
            height: 25,
        }))
    );
}

#[test]
fn tui_host_process_next_resize_without_handler_returns_tag_four() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.push_console_event(ConsoleEvent::resize(5, 5));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("A"),
        'a',
        false,
        false,
        false,
        false,
    )));
    vm.run().expect("vm ok");
    assert_eq!(vm.output().lines, vec!["4"]);
}

#[test]
fn tui_host_process_next_resize_without_handler_still_marks_union_of_surface_bounds() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::resize(90, 30));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )));

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
            x: 0,
            y: 0,
            width: 90,
            height: 30,
        }))
    );
}

#[test]
fn tui_host_process_next_resize_to_smaller_surface_preserves_old_bounds_in_damage() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::resize(40, 10));
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
            key_kind_index("Escape"),
            '\u{1b}',
            false,
            false,
            false,
            false,
        )));

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
            x: 0,
            y: 0,
            width: 80,
            height: 25,
        }))
    );
}

#[test]
fn tui_host_dispatch_redraw_invokes_on_paint() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
fn tui_host_register_on_idle_stores_handler_and_interval() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(25));
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
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert!(matches!(
        tui.on_idle.as_ref(),
        Some(Value::Function { name, .. }) if name == "OnIdle"
    ));
    assert_eq!(tui.idle_interval_ms, 25);
}

#[test]
fn tui_host_register_on_idle_clamps_negative_interval_to_zero() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(-5));
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
    chunk.emit(Op::Halt, loc());

    let on_idle_start = chunk.len();
    chunk.functions.insert("OnIdle".into(), (on_idle_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    let mut worker = Worker::new_main(Arc::clone(&shared));
    worker.run().expect("VM should succeed");

    let tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(tui.idle_interval_ms, 0);
}

#[test]
fn tui_host_dispatch_redraw_without_handler_clears_and_returns_six() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
fn tui_host_run_loop_dispatches_paint_then_key_until_idle() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit))), loc());
    emit_constant(&mut chunk, Value::Integer(10_000));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRunLoop))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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

#[test]
fn tui_host_register_on_mouse_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouseRedraw".into(), (on_mouse_start, 2));
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
    chunk.emit(Op::Halt, loc());

    let on_mouse_start = chunk.len();
    chunk
        .functions
        .insert("OnMouse".into(), (on_mouse_start, 2));
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk
        .functions
        .insert("WrongOnMouse".into(), (on_mouse_start, 1));
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let error = run_err(chunk);
    assert!(
        error.message.contains("OnMouse handler must have arity 2"),
        "unexpected runtime error: {}",
        error.message
    );
}

// ── OnPaste ──────────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_paste_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_paste_start = chunk.len();
    chunk
        .functions
        .insert("OnPasteRedraw".into(), (on_paste_start, 2));
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
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

// ── OnFocusGained ─────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_focus_gained_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGainedRedraw".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_fg_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusGainedRedraw".into(), (on_fg_start, 2));
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusGained".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFG".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained))),
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

// ── OnFocusLost ───────────────────────────────────────────────────────────────

#[test]
fn tui_host_register_on_focus_lost_stores_handler_in_shared_tui_state() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
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
fn tui_host_focus_lost_redraw_hint_uses_focused_view_rect_when_handler_requests_redraw() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLostRedraw".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLostRedraw".into(), (on_fl_start, 2));
    chunk.emit(Op::GetLocal(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw))),
        loc(),
    );
    emit_constant(&mut chunk, Value::Unit);
    chunk.emit(Op::Return, loc());

    let shared = Arc::new(minimal_shared_state(chunk));
    {
        let mut tui = shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        let view_id = tui.views.register(ViewRect {
            x: 12,
            y: 6,
            width: 3,
            height: 2,
        });
        tui.views.push_child(view_id);
        let _ = tui.views.focus_next();
    }
    shared
        .key_input
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push_console_event(ConsoleEvent::focus_lost());

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
            x: 12,
            y: 6,
            width: 3,
            height: 2,
        }))
    );
}

#[test]
fn tui_host_process_next_focus_lost_without_handler_returns_tag_thirteen() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(&mut chunk, tui_application_value());
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))), loc());
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "OnFocusLost".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost))),
        loc(),
    );
    emit_constant(&mut chunk, tui_application_value());
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationClose))), loc());
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk
        .functions
        .insert("OnFocusLost".into(), (on_fl_start, 2));
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
    chunk.emit(Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "WrongFL".into(),
            captures: vec![],
        },
    );
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let on_fl_start = chunk.len();
    chunk.functions.insert("WrongFL".into(), (on_fl_start, 1));
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
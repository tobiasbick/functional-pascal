//! VM-level tests for Phase 7 Step 2: host-managed focus chain, Tab/Shift+Tab traversal,
//! and `OnActivate`/`OnDeactivate` dispatch.
//!
//! These tests populate `TuiState.views` directly from Rust where convenient, and also cover
//! the additive FPAS-facing host view API introduced after the initial focus-chain step.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, DamageRegion, ViewRect, key_event::key_kind_index};
use std::sync::Arc;

use crate::tests::helpers::{emit_constant, loc, minimal_shared_state};
use crate::vm::Worker;

mod traversal;
mod view_chain;

fn tab_event(shift: bool) -> ConsoleEvent {
    ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Tab"),
        '\t',
        shift,
        false,
        false,
        false,
    ))
}

fn view_rect() -> ViewRect {
    ViewRect {
        x: 0,
        y: 0,
        width: 10,
        height: 5,
    }
}

fn view_rect_at(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
    ViewRect {
        x,
        y,
        width,
        height,
    }
}

fn build_process_next_chunk_with_handlers(
    on_activate_name: Option<&str>,
    on_deactivate_name: Option<&str>,
    on_key_name: Option<&str>,
) -> Chunk {
    let mut chunk = Chunk::new();

    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen))),
        loc(),
    );

    if let Some(name) = on_activate_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(u16::from(Intrinsic::Tui(
                TuiIntrinsic::HostRegisterOnActivate,
            ))),
            loc(),
        );
    }
    if let Some(name) = on_deactivate_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(u16::from(Intrinsic::Tui(
                TuiIntrinsic::HostRegisterOnDeactivate,
            ))),
            loc(),
        );
    }
    if let Some(name) = on_key_name {
        chunk.emit(Op::Dup, loc());
        emit_constant(
            &mut chunk,
            Value::Function {
                name: name.into(),
                captures: vec![],
            },
        );
        chunk.emit(
            Op::Intrinsic(u16::from(Intrinsic::Tui(
                TuiIntrinsic::HostRegisterOnKeyPressed,
            ))),
            loc(),
        );
    }

    emit_constant(&mut chunk, Value::Integer(64));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Tui(TuiIntrinsic::HostProcessNext))),
        loc(),
    );
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    chunk
}

fn add_handler(chunk: &mut Chunk, name: &str, arity: u8, body_output: &str) {
    let start = chunk.len();
    chunk.functions.insert(name.to_string(), (start, arity));
    emit_constant(chunk, Value::Str(body_output.into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(chunk, Value::Unit);
    chunk.emit(Op::Return, loc());
}

fn add_key_handler(chunk: &mut Chunk, name: &str, output: &str) {
    let start = chunk.len();
    chunk.functions.insert(name.to_string(), (start, 2));
    emit_constant(chunk, Value::Str(output.into()));
    chunk.emit(Op::PrintLn, loc());
    emit_constant(chunk, Value::Boolean(true));
    chunk.emit(Op::Return, loc());
}

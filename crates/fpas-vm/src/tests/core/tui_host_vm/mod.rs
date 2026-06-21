//! VM bridge for `TuiHost` (Phase 3): host poll/register/process intrinsics (see `docs/pascal/std/tui/app/README.md`).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

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
    tui_application_value, tui_view_id_value,
};
use crate::vm::Worker;

mod commands;
mod focus_gained;
mod focus_lost;
mod frame_views;
mod idle_registration;
mod modal_bindings;
mod modal_views;
mod mouse;
mod paste;
mod poll_resize;
mod redraw;
mod runloop_lifecycle;
mod views;

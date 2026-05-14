//! VM tests for `Std.Tui.Application.Run`.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use fpas_bytecode::{Chunk, Intrinsic, Op, TuiIntrinsic, Value};
use fpas_std::ConsoleEvent;
use std::sync::Arc;
use std::thread;

use crate::tests::helpers::{
    emit_constant, loc, minimal_shared_state, run_err, tui_application_value,
};
use crate::vm::Worker;

mod idle;
mod lifecycle;
mod resize;
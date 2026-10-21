//! Direct register-interpreter regression tests.

mod aggregates;
mod callbacks;
mod calls;
mod closure_calls;
mod console_eof;
mod dispatch_budget;
mod global_index_path;
mod inline_callbacks;
mod intrinsics;
mod register_initialization;
mod register_stack;
mod runtime;
mod scalar;
mod shutdown;
mod superinstructions;
mod support;
/// Ownership-aware assembler for hand-built task fixtures.
pub(in crate::vm) mod task_images;
mod task_owned_functions;
mod task_scopes;

use support::*;

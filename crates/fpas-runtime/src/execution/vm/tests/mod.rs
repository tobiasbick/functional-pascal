//! Direct register-interpreter regression tests.

mod aggregate_reads;
mod aggregates;
mod callback_frames;
mod callbacks;
mod calls;
mod cleanup;
mod cleanup_compiler;
mod closure_calls;
mod console_eof;
mod dispatch_budget;
mod dynamic;
mod global_index_path;
mod inline_callbacks;
mod intrinsic_length;
mod intrinsic_unwrap;
mod intrinsics;
mod real_arithmetic;
mod register_initialization;
mod register_stack;
mod runtime;
mod scalar;
mod shutdown;
mod superinstructions;
mod support;
/// Ownership-aware assembler for hand-built task fixtures.
pub(in crate::execution::vm) mod task_images;
mod task_owned_functions;
mod task_scopes;
mod wrappers;

use support::*;

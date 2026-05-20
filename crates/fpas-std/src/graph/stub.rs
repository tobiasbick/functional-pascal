//! `Std.Graph` runtime stub diagnostics for the current VM integration slice.
//!
//! **Documentation:** `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{GraphIntrinsic, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Executes the current `Std.Graph` runtime stub.
///
/// All Phase 1 graph intrinsics currently fail with a clear runtime diagnostic until
/// the VM value bridge and native backend are implemented.
pub fn run_graph_intrinsic(
    intrinsic: GraphIntrinsic,
    location: SourceLocation,
) -> Result<(), StdError> {
    Err(not_implemented_error(intrinsic, location))
}

fn not_implemented_error(intrinsic: GraphIntrinsic, location: SourceLocation) -> StdError {
    let routine = match intrinsic {
        GraphIntrinsic::ApplicationOpen => "Std.Graph.Application.Open",
        GraphIntrinsic::ApplicationClose => "Std.Graph.Application.Close",
        GraphIntrinsic::ApplicationSize => "Std.Graph.Application.Size",
        GraphIntrinsic::ApplicationPollEvent => "Std.Graph.Application.PollEvent",
        GraphIntrinsic::ApplicationUploadFrame => "Std.Graph.Application.UploadFrame",
    };

    std_runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!("{routine} is not implemented yet."),
        "Std.Graph compiles, but the current runtime slice still uses a stub. Implement the VM value bridge and native backend before running this call.",
        location,
    )
}
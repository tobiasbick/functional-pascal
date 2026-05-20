//! `Std.Graph` Phase 1 runtime stub.
//!
//! **Documentation:** `docs/future/std.graph/01-mvp.md`, `docs/future/std.graph/02-pascal-surface.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{GraphIntrinsic, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

/// Canonical `Std.Graph.EventKind` variant names for semantic registration and short aliases.
///
/// **Documentation:** `docs/future/std.graph/02-pascal-surface.md` (from the repository root).
pub const GRAPH_EVENT_KIND_VARIANTS: &[&str] = &["CloseRequested", "Resize", "Key"];

/// Executes the current `Std.Graph` runtime stub.
///
/// All Phase 1 graph intrinsics currently fail with a clear runtime diagnostic until
/// the native window backend and VM value bridge are implemented.
///
/// **Documentation:** `docs/future/std.graph/04-implementation-plan.md` (from the repository root).
pub fn run_graph_intrinsic(intrinsic: GraphIntrinsic, location: SourceLocation) -> Result<(), StdError> {
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
        "Std.Graph compiles, but the VM/runtime slice is still a stub. Implement the planned VM bridge and native runtime support before running this call.",
        location,
    )
}

#[cfg(test)]
mod tests {
    use super::run_graph_intrinsic;
    use fpas_bytecode::{GraphIntrinsic, SourceLocation};

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn application_open_reports_not_implemented_yet() {
        let err = run_graph_intrinsic(GraphIntrinsic::ApplicationOpen, loc()).expect_err("expected stub error");
        assert!(
            err.message.contains("Std.Graph.Application.Open is not implemented yet"),
            "message={}",
            err.message
        );
    }
}
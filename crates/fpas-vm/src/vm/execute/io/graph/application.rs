//! `Std.Graph` application stub intrinsics.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation};

impl Worker {
    /// Executes application-level `Std.Graph` intrinsics through the current stub runtime.
    pub(super) fn try_exec_graph_application_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        let graph_intrinsic = match intrinsic {
            Intrinsic::Graph(GraphIntrinsic::ApplicationOpen) => GraphIntrinsic::ApplicationOpen,
            Intrinsic::Graph(GraphIntrinsic::ApplicationClose) => GraphIntrinsic::ApplicationClose,
            Intrinsic::Graph(GraphIntrinsic::ApplicationSize) => GraphIntrinsic::ApplicationSize,
            Intrinsic::Graph(GraphIntrinsic::ApplicationPollEvent) => {
                GraphIntrinsic::ApplicationPollEvent
            }
            Intrinsic::Graph(GraphIntrinsic::ApplicationUploadFrame) => {
                GraphIntrinsic::ApplicationUploadFrame
            }
            _ => return Ok(false),
        };

        fpas_std::run_graph_intrinsic(graph_intrinsic, line)?;
        Ok(true)
    }
}
//! Lowers `Std.Graph` Phase 1 calls to VM intrinsics.
//!
//! **Documentation:** `docs/future/std.graph/02-pascal-surface.md`, `docs/future/std.graph/04-implementation-plan.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::intrinsic::GraphIntrinsic;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Lower `Std.Graph.Application.*` Phase 1 calls.
    pub(super) fn compile_graph_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_GRAPH_APPLICATION_OPEN => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_OPEN, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Graph(GraphIntrinsic::ApplicationOpen), location);
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_CLOSE => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_CLOSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationClose),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_SIZE => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_SIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Graph(GraphIntrinsic::ApplicationSize), location);
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_POLL_EVENT => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_POLL_EVENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationPollEvent),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_UPLOAD_FRAME => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_UPLOAD_FRAME, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationUploadFrame),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

//! `Std.Graph` host-control intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph/app/README.md` (from the repository root).

mod lifecycle;
mod process;
mod redraw;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation};

impl Worker {
    /// Executes host-control `Std.Graph` intrinsics.
    pub(super) fn try_exec_graph_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnKeyPressed) => {
                self.register_graph_handler(
                    2,
                    "OnKeyPressed",
                    "Pass a `function (Application, Std.Console.KeyEvent): boolean`.",
                    |graph, function| graph.on_key_pressed = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnResize) => {
                self.register_graph_handler(
                    2,
                    "OnResize",
                    "Pass a `procedure (Application, Std.Graph.Size)`.",
                    |graph, function| graph.on_resize = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostProcessNext) => {
                let max_spins = self.pop_int(line)?.clamp(0, 4096) as usize;
                self.pop_graph_application(line)?;
                let tag = self.graph_host_process_next_inner(max_spins, line)?;
                self.push(fpas_bytecode::Value::Integer(tag))?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnPaint) => {
                self.register_graph_handler(
                    1,
                    "OnPaint",
                    "Pass a `procedure (Application)`.",
                    |graph, function| graph.on_paint = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnIdle) => {
                let function = self.pop(line)?;
                let milliseconds = self.pop_int(line)?.max(0);
                self.pop_graph_application(line)?;
                self.validate_host_handler_function(
                    &function,
                    1,
                    "OnIdle",
                    "Pass `Application`, an idle interval in milliseconds, and a `procedure (Application)` handler.",
                    line,
                )?;
                self.with_graph(|graph| {
                    graph.on_idle = Some(function);
                    graph.idle_interval_ms = milliseconds;
                });
            }
            Intrinsic::Graph(GraphIntrinsic::HostDispatchRedraw) => {
                self.pop_graph_application(line)?;
                let tag = self.graph_host_dispatch_redraw_inner(line)?;
                self.push(fpas_bytecode::Value::Integer(tag))?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRequestQuit) => {
                self.pop_graph_application(line)?;
                self.with_graph(|graph| graph.quit_requested = true);
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnExit) => {
                self.register_graph_handler(
                    2,
                    "OnExit",
                    "Pass a `procedure (Application, Std.Graph.ExitReason)`.",
                    |graph, function| graph.on_exit = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnMouse) => {
                self.register_graph_handler(
                    2,
                    "OnMouse",
                    "Pass a `procedure (Application, Std.Graph.Event)`.",
                    |graph, function| graph.on_mouse = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnWheel) => {
                self.register_graph_handler(
                    2,
                    "OnWheel",
                    "Pass a `procedure (Application, Std.Graph.Event)`.",
                    |graph, function| graph.on_wheel = Some(function),
                    line,
                )?;
            }
            Intrinsic::Graph(GraphIntrinsic::HostRegisterOnCloseRequested) => {
                self.register_graph_handler(
                    1,
                    "OnCloseRequested",
                    "Pass a `procedure (Application)`.",
                    |graph, function| graph.on_close_requested = Some(function),
                    line,
                )?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}

//! Lowers `Std.Graph` Phase 1 calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph/session.md` (from the repository root).

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
            s::STD_GRAPH_APPLICATION_REQUEST_REDRAW => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_REQUEST_REDRAW, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationRequestRedraw),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_CONFIGURE => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_CONFIGURE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationConfigure),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_RUN => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_RUN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationRun),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REQUEST_QUIT,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRequestQuit),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnKeyPressed),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_RESIZE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnResize),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_PROCESS_NEXT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Graph(GraphIntrinsic::HostProcessNext), location);
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_PAINT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnPaint),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_IDLE,
                    3,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnIdle),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_DISPATCH_REDRAW,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(
                    Intrinsic::Graph(GraphIntrinsic::HostDispatchRedraw),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_EXIT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnExit),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_MOUSE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnMouse),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_WHEEL,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnWheel),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED => {
                self.expect_exact_args(
                    s::STD_GRAPH_APPLICATION_HOST_REGISTER_ON_CLOSE_REQUESTED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::HostRegisterOnCloseRequested),
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
            s::STD_GRAPH_APPLICATION_CLEAR => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_CLEAR, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationClear),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_PUT_PIXEL => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_PUT_PIXEL, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationPutPixel),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_PRESENT => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_PRESENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationPresent),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_DRAW_LINE => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_DRAW_LINE, 6, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawLine),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_DRAW_RECT => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_DRAW_RECT, 6, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawRect),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_FILL_RECT => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_FILL_RECT, 6, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationFillRect),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_DRAW_CIRCLE => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_DRAW_CIRCLE, 5, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawCircle),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_DRAW_TEXT => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_DRAW_TEXT, 5, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Graph(GraphIntrinsic::ApplicationDrawText),
                    location,
                );
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_OPEN_FOR_TEST => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_OPEN_FOR_TEST, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Graph(GraphIntrinsic::OpenForTest), location);
                Ok(true)
            }
            s::STD_GRAPH_APPLICATION_TEST_SEND_KEY => {
                self.expect_exact_args(s::STD_GRAPH_APPLICATION_TEST_SEND_KEY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Graph(GraphIntrinsic::TestSendKey), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

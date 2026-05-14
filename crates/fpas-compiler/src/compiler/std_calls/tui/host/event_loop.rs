use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*` event-loop calls.
    pub(super) fn compile_tui_host_event_loop_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_HOST_POLL_NEXT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_POLL_NEXT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostPollNext), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostProcessNext), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_RUN_LOOP => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_RUN_LOOP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRunLoop), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*` input and focus callback calls.
    pub(super) fn compile_tui_host_input_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

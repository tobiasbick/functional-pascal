use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*Command*` calls.
    pub(super) fn compile_tui_host_command_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_BIND_COMMAND => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_BIND_COMMAND, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostBindCommand), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

use crate::error::CompileError;
use fpas_bytecode::SourceLocation;
use fpas_parser::Expr;

mod command;
mod event_loop;
mod input;
mod lifecycle;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*` calls.
    pub(super) fn compile_tui_host_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        if self.compile_tui_host_event_loop_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_host_input_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_host_lifecycle_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_host_command_call(name, args, location)? {
            return Ok(true);
        }
        Ok(false)
    }
}

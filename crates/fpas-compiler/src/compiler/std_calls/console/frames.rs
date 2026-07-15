//! Lowering for framed drawing and saved console regions.

use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` frame and saved-region calls.
    pub(super) fn compile_console_frame_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let (arity, intrinsic, returns_value) = match name {
            s::STD_CONSOLE_BEGIN_FRAME => (0, ConsoleIntrinsic::BeginFrame, false),
            s::STD_CONSOLE_PRESENT => (0, ConsoleIntrinsic::Present, false),
            s::STD_CONSOLE_SAVE_REGION => (1, ConsoleIntrinsic::SaveRegion, true),
            s::STD_CONSOLE_RESTORE_REGION => (1, ConsoleIntrinsic::RestoreRegion, false),
            s::STD_CONSOLE_DISCARD_REGION => (1, ConsoleIntrinsic::DiscardRegion, false),
            _ => return Ok(false),
        };

        self.expect_exact_args(name, arity, args, location)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        let intrinsic = Intrinsic::Console(intrinsic);
        if returns_value {
            self.emit_intrinsic(intrinsic, location);
        } else {
            self.emit_intrinsic_unit(intrinsic, location);
        }
        Ok(true)
    }
}

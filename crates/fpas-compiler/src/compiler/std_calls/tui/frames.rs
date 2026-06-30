//! Lower frame-root dialog and query calls.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower one frame-root call when `name` belongs to the remaining frame API.
    pub(super) fn compile_tui_frame_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let (arity, intrinsic) = match name {
            s::STD_TUI_APPLICATION_SHOW_FRAMED_DIALOG => {
                (12, TuiIntrinsic::ApplicationShowFramedDialog)
            }
            s::STD_TUI_APPLICATION_QUERY_FRAME_ROOT_STATE => (2, TuiIntrinsic::QueryFrameRootState),
            s::STD_TUI_APPLICATION_QUERY_FRAME_SCROLL_STATE => {
                (2, TuiIntrinsic::QueryFrameScrollState)
            }
            s::STD_TUI_APPLICATION_QUERY_FRAME_WINDOW_LIST => {
                (1, TuiIntrinsic::QueryFrameWindowList)
            }
            _ => return Ok(false),
        };
        self.expect_exact_args(name, arity, args, location)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        self.emit_intrinsic(Intrinsic::Tui(intrinsic), location);
        Ok(true)
    }
}

//! Lower frame-root host calls.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower one frame-root call when `name` belongs to the frame host API.
    pub(super) fn compile_tui_frame_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let (arity, intrinsic, returns_value) = match name {
            s::STD_TUI_APPLICATION_HOST_SET_DESKTOP_WORK_AREA => {
                (5, TuiIntrinsic::HostSetDesktopWorkArea, true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_FRAME_VIEW => {
                (11, TuiIntrinsic::HostCreateFrameView, true)
            }
            s::STD_TUI_APPLICATION_SHOW_FRAMED_DIALOG => {
                (11, TuiIntrinsic::ApplicationShowFramedDialog, true)
            }
            s::STD_TUI_APPLICATION_HOST_ACTIVATE_NEXT_WINDOW => {
                (1, TuiIntrinsic::HostActivateNextWindow, true)
            }
            s::STD_TUI_APPLICATION_HOST_ZOOM_FRAME_ROOT => {
                (2, TuiIntrinsic::HostZoomFrameRoot, true)
            }
            s::STD_TUI_APPLICATION_HOST_RESTORE_FRAME_ROOT => {
                (2, TuiIntrinsic::HostRestoreFrameRoot, true)
            }
            s::STD_TUI_APPLICATION_QUERY_FRAME_ROOT_STATE => {
                (2, TuiIntrinsic::QueryFrameRootState, true)
            }
            s::STD_TUI_APPLICATION_HOST_CASCADE_FRAME_ROOTS => {
                (3, TuiIntrinsic::HostCascadeFrameRoots, true)
            }
            s::STD_TUI_APPLICATION_HOST_TILE_FRAME_ROOTS => {
                (1, TuiIntrinsic::HostTileFrameRoots, true)
            }
            _ => return Ok(false),
        };
        self.expect_exact_args(name, arity, args, location)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        if returns_value {
            self.emit_intrinsic(Intrinsic::Tui(intrinsic), location);
        } else {
            self.emit_intrinsic_unit(Intrinsic::Tui(intrinsic), location);
        }
        Ok(true)
    }
}

//! Lower retained TUI control calls.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower one control call when `name` belongs to the retained control API.
    pub(super) fn compile_tui_control_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let (arity, intrinsic, returns_value) = match name {
            s::STD_TUI_APPLICATION_HOST_CREATE_LABEL_VIEW => {
                (7, TuiIntrinsic::HostCreateLabelView, true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_BUTTON_VIEW => {
                (8, TuiIntrinsic::HostCreateButtonView, true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_INPUT_LINE_VIEW => {
                (6, TuiIntrinsic::HostCreateInputLineView, true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_CHECK_BOX_VIEW => {
                (9, TuiIntrinsic::HostCreateCheckBoxView, true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_RADIO_GROUP_VIEW => {
                (6, TuiIntrinsic::HostCreateRadioGroupView, true)
            }
            s::STD_TUI_APPLICATION_HOST_SET_INPUT_LINE_TEXT => {
                (3, TuiIntrinsic::HostSetInputLineText, false)
            }
            s::STD_TUI_APPLICATION_HOST_SET_CHECK_BOX_CHECKED => {
                (3, TuiIntrinsic::HostSetCheckBoxChecked, false)
            }
            s::STD_TUI_APPLICATION_HOST_SET_RADIO_GROUP_SELECTED => {
                (3, TuiIntrinsic::HostSetRadioGroupSelected, false)
            }
            s::STD_TUI_APPLICATION_QUERY_INPUT_LINE_STATE => {
                (2, TuiIntrinsic::QueryInputLineState, true)
            }
            s::STD_TUI_APPLICATION_QUERY_CHECK_BOX_STATE => {
                (2, TuiIntrinsic::QueryCheckBoxState, true)
            }
            s::STD_TUI_APPLICATION_QUERY_RADIO_GROUP_STATE => {
                (2, TuiIntrinsic::QueryRadioGroupState, true)
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

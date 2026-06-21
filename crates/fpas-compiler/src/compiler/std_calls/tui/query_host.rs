use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower native TUI screen query calls.
    pub(super) fn compile_tui_query_host_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_QUERY_SCREEN_SIZE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_SCREEN_SIZE,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryScreenSize), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_SCREEN_LINE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_SCREEN_LINE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryScreenLine), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_SCREEN_CELL => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_SCREEN_CELL,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryScreenCell), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_ROOT_VIEWS => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUERY_ROOT_VIEWS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryRootViews), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_RECT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUERY_VIEW_RECT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewRect), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_PARENT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_VIEW_PARENT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewParent), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_CHILDREN => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_VIEW_CHILDREN,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewChildren), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_STATE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUERY_VIEW_STATE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewState), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_OPTIONS => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_VIEW_OPTIONS,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewOptions), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_RESOLVED_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_RESOLVED_VIEW,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryResolvedView), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_VIEW_KIND => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUERY_VIEW_KIND, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryViewKind), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_SCENE_GRAPH => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_SCENE_GRAPH,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QuerySceneGraph), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_MENU_BAR_STATE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_MENU_BAR_STATE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryMenuBarState), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_MODAL_DEPTH => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_MODAL_DEPTH,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryModalDepth), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUERY_FOCUSED_VIEW_ID => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_QUERY_FOCUSED_VIEW_ID,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::QueryFocusedViewId), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

//! `Std.Tui` VM execution — intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod application;
mod callbacks;
mod commands;
mod controls;
mod dialogs;
mod events;
mod file_dialog;
mod handlers;
mod handles;
mod host;
mod navigation;
mod query_host;
mod records;
mod testing;
mod tv_geometry;
mod tv_run;
mod windows;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};

impl Worker {
    /// Execute a `Std.Tui` intrinsic in the VM.
    pub(super) fn try_exec_tui_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        if self.try_exec_tui_application_intrinsic(intrinsic, line)?
            || self.try_exec_turbo_vision_intrinsic(intrinsic, line)?
            || self.try_exec_tui_test_host_intrinsic(intrinsic, line)?
            || self.try_exec_tui_query_host_intrinsic(intrinsic, line)?
            || self.try_exec_tui_host_intrinsic(intrinsic, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Execute Turbo Vision backed `Std.Tui` spike intrinsics.
    pub(super) fn try_exec_turbo_vision_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::CreateDialog) => {
                self.turbo_vision_create_dialog(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateButton) => {
                self.turbo_vision_create_button(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateStaticText) => {
                self.turbo_vision_create_static_text(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateMemo) => {
                self.turbo_vision_create_memo(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateTextViewer) => {
                self.turbo_vision_create_text_viewer(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateInputLine) => {
                self.turbo_vision_create_input_line(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateListBox) => {
                self.turbo_vision_create_list_box(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateCheckBox) => {
                self.turbo_vision_create_check_box(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateRadioButton) => {
                self.turbo_vision_create_radio_button(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RunFileDialog) => {
                self.turbo_vision_run_file_dialog(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::AddChild) => {
                self.turbo_vision_add_child(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand) => {
                self.turbo_vision_register_on_command(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::Pump) => {
                self.turbo_vision_pump(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::Quit) => {
                self.turbo_vision_quit(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::TestClickButton) => {
                self.turbo_vision_test_click_button(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateWindow) => {
                self.turbo_vision_create_window(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::AddWindow) => {
                self.turbo_vision_add_window(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateMenuBar) => {
                self.turbo_vision_create_menu_bar(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::SetMenuBar) => {
                self.turbo_vision_set_menu_bar(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::CreateStatusLine) => {
                self.turbo_vision_create_status_line(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::SetStatusLine) => {
                self.turbo_vision_set_status_line(line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}

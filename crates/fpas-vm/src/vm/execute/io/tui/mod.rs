//! `Std.Tui` VM execution — intrinsic dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod application;
mod bridged_button;
mod bridged_check_box;
mod bridged_list_box;
mod bridged_memo;
mod bridged_outline;
mod bridged_radio_button;
mod bridged_static_text;
mod bridged_text_viewer;
mod chrome_layout;
mod command_map;
mod commands;
mod handle_records;
mod handles;
mod headless_tv_draw;
mod lifecycle;
mod navigation;
mod outline_nodes;
mod records;
mod session_app;
mod testing;
mod try2;

pub(in crate::vm) use try2::Try2Session;
mod tv_geometry;
mod tv_headless_backend;
mod tv_input_events;

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
            || self.try_exec_try2_intrinsic(intrinsic, line)?
            || self.try_exec_turbo_vision_intrinsic(intrinsic, line)?
            || self.try_exec_tui_test_host_intrinsic(intrinsic, line)?
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Dispatch shared application chrome, modal, and test intrinsics on the try-2 path.
    pub(super) fn try_exec_turbo_vision_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::RunFileDialog) => {
                self.turbo_vision_run_file_dialog(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::MessageBox) => {
                self.turbo_vision_message_box(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand) => {
                self.turbo_vision_register_on_command(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RegisterOnKey) => {
                self.turbo_vision_register_on_key(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::RegisterOnMouse) => {
                self.turbo_vision_register_on_mouse(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::Quit) => {
                self.turbo_vision_quit(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::TestClickButton) => {
                self.exec_test_click_button(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::TestClickMouse) => {
                self.turbo_vision_test_click_mouse(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::TestDispatchMenuCommand) => {
                self.turbo_vision_test_dispatch_menu_command(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::SetMenuBar) => {
                self.turbo_vision_set_menu_bar(line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::SetStatusLine) => {
                self.turbo_vision_set_status_line(line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}

pub(in crate::vm) use headless_tv_draw::HeadlessTvApp;

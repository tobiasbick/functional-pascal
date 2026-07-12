//! Turbo Vision bridge `Application.MessageBox` on the live or headless turbo-vision session.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::app::bridge_ensure_live_app;
use super::chrome::bridge_sync_chrome_to_app;
use super::headless::{bridge_ensure_headless_app, bridge_headless_exec_view};
use super::headless_draw::HeadlessTvApp;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::helpers::msgbox::message_box;
use turbo_vision::helpers::msgbox::{
    MF_ABOUT, MF_CANCEL_BUTTON, MF_CONFIRMATION, MF_ERROR, MF_INFORMATION, MF_NO_BUTTON,
    MF_OK_BUTTON, MF_WARNING, MF_YES_BUTTON,
};
use turbo_vision::views::View;
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::static_text::StaticText;

/// Shows a message box when the Turbo Vision session is active.
pub(in crate::vm::execute::io::tui) fn bridge_message_box(
    worker: &mut Worker,
    message: String,
    options: u16,
    line: SourceLocation,
) -> Result<i64, VmError> {
    if worker.current_task_id != 0 {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.MessageBox(App, ...) must run on the main task",
            "Call `Application.MessageBox` from the main program, not from a `go` task.",
            line,
        ));
    }

    bridge_sync_chrome_to_app(worker, line)?;

    let command = if worker.with_tui(|tui| tui.session.is_headless()) {
        bridge_headless_message_box(worker, &message, options, line)?
    } else {
        bridge_ensure_live_app(worker, line)?;
        let Some(app) = worker.live_turbo_vision_app.as_mut() else {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Turbo Vision live session is not initialized",
                "Call `Application.Open` before `Application.MessageBox`.",
                line,
            ));
        };
        message_box(app, &message, options)
    };

    Ok(i64::from(command))
}

fn bridge_headless_message_box(
    worker: &mut Worker,
    message: &str,
    options: u16,
    line: SourceLocation,
) -> Result<CommandId, VmError> {
    if let Some(command) = worker.bridge.take_dialog_result() {
        return Ok(u16::try_from(command).map_err(|_| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.TestSetDialogResult command must fit in 16 bits",
                "Pass a non-negative command id such as `CM_OK` or a custom widget command.",
                line,
            )
        })?);
    }

    bridge_ensure_headless_app(worker, line)?;
    let (width, height) = worker
        .headless_tv_app
        .as_ref()
        .map(HeadlessTvApp::terminal_size)
        .unwrap_or((80, 25));
    let bounds = message_box_bounds(message, width, height);
    let dialog = build_message_box_dialog(bounds, message, options);
    let command = bridge_headless_exec_view(worker, dialog, line)?;
    worker.turbo_vision_export_headless_to_console();
    Ok(command)
}

fn message_box_bounds(message: &str, width: i16, height: i16) -> Rect {
    let lines: Vec<&str> = message.split('\n').collect();
    let num_lines = lines.len() as i16;
    let max_line_len = lines.iter().map(|line| line.len()).max().unwrap_or(0) as i16;
    let dialog_width = (max_line_len + 4).clamp(40, 72);
    let dialog_height = (1 + num_lines + 2 + 3).clamp(9, 20);
    let dialog_x = (width - dialog_width) / 2;
    let dialog_y = (height - dialog_height - 2) / 2;
    Rect::new(
        dialog_x,
        dialog_y,
        dialog_x + dialog_width,
        dialog_y + dialog_height,
    )
}

fn build_message_box_dialog(bounds: Rect, message: &str, options: u16) -> Box<dyn View> {
    let title = match options & 0x0F {
        MF_WARNING => "Warning ⚠️",
        MF_ERROR => "Error 🛑",
        MF_INFORMATION => "Information ℹ️",
        MF_CONFIRMATION => "❔Confirm❔",
        MF_ABOUT => "About ℹ️",
        _ => "Message",
    };

    let mut dialog = Dialog::new_modal(bounds, title);
    let text_bounds = Rect::new(1, 1, bounds.width() - 2, bounds.height() - 3);
    dialog.add(Box::new(StaticText::new(text_bounds, message)));

    let button_specs = [
        (MF_YES_BUTTON, "~Y~es", CM_YES),
        (MF_NO_BUTTON, "~N~o", CM_NO),
        (MF_OK_BUTTON, "O~K~", CM_OK),
        (MF_CANCEL_BUTTON, "Cancel", CM_CANCEL),
    ];

    let mut buttons = Vec::new();
    let mut total_width = -2i16;
    for (flag, label, command) in button_specs.iter() {
        if (options & flag) != 0 {
            let button = Button::new(Rect::new(0, 0, 10, 2), label, *command, buttons.is_empty());
            total_width += 10 + 2;
            buttons.push((button, *command));
        }
    }

    let mut x = (bounds.width() - total_width) / 2;
    let y = bounds.height() - 4;
    for (mut button, _) in buttons {
        button.set_bounds(Rect::new(x, y, x + 10, y + 2));
        dialog.add(Box::new(button));
        x += 12;
    }

    dialog.set_initial_focus();
    dialog
}

impl Worker {
    /// Show an upstream Turbo Vision message box and push the closing command id.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_message_box(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let options = u16::try_from(self.pop_int(line)?).map_err(|_| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.MessageBox options must fit in 16 bits",
                "Pass a non-negative options value such as `MessageBoxOption.About + MessageBoxOption.OkButton`.",
                line,
            )
        })?;
        let message = self.pop_turbo_vision_string("MessageBox Message", line)?;
        self.pop_tui_application(line)?;

        if !self.bridge.is_open() {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.MessageBox requires an open Turbo Vision session",
                "Call `Application.Open` or `Application.OpenForTest` before `Application.MessageBox`.",
                line,
            ));
        }

        let command = bridge_message_box(self, message, options, line)?;
        self.push(Value::Integer(command))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::{loc, minimal_shared_state};
    use crate::vm::Worker;
    use fpas_bytecode::Chunk;
    use std::sync::Arc;
    use turbo_vision::core::event::KB_ENTER;
    use turbo_vision::helpers::msgbox::{MF_ABOUT, MF_OK_BUTTON};

    fn headless_bridge_worker(width: u16, height: u16) -> Worker {
        let shared = Arc::new(minimal_shared_state(Chunk::new()));
        let mut worker = Worker::new_main(shared);
        worker.with_console(|console| console.resize(width, height));
        {
            let mut tui = worker.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            worker.with_console(|console| {
                tui.session
                    .open_for_test(console, loc())
                    .expect("open_for_test");
            });
        }
        worker.open_bridge_session();
        worker
    }

    #[test]
    fn headless_message_box_ok_returns_cm_ok() {
        let mut worker = headless_bridge_worker(60, 20);
        bridge_ensure_headless_app(&mut worker, loc()).expect("headless app");
        worker
            .headless_tv_app
            .as_ref()
            .expect("headless app")
            .push_keyboard(KB_ENTER);
        let command =
            bridge_message_box(&mut worker, "Hello".into(), MF_ABOUT | MF_OK_BUTTON, loc())
                .expect("message box");
        assert_eq!(command, i64::from(CM_OK));
    }
}

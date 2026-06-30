//! Turbo Vision application-level callback and test-pump operations.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (Turbo Vision spike API).

use super::widgets::unknown_handle_error;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionButton, TurboVisionObject, TurboVisionRect};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_std::ProcessOutcome;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::core::event::Event;
use turbo_vision::views::{button::Button, dialog::Dialog};

const HEADLESS_RUN_MAX_COMMANDS: usize = 4096;

impl Worker {
    pub(super) fn turbo_vision_register_on_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.register_tui_handler(
            2,
            "OnCommand",
            "Pass a `procedure (Application, integer)` command handler.",
            |tui, function| tui.on_command = Some(function),
            line,
        )?;
        Ok(())
    }

    pub(super) fn turbo_vision_test_click_button(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        let button_handle = self.pop_turbo_vision_button_handle(line)?;
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            let Some(TurboVisionObject::Button(button)) =
                tui.turbo_vision.objects.get(&button_handle)
            else {
                return Err(unknown_handle_error("Button", button_handle, line));
            };
            tui.turbo_vision
                .pending_commands
                .push_back(button.command_id);
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn turbo_vision_pump(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.pop_tui_application(line)?;
        let outcome = self.turbo_vision_pump_next_command(line)?;
        self.push(Value::Integer(outcome.bridge_tag()))
    }

    fn turbo_vision_pump_next_command(
        &mut self,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        let command = self.with_tui(|tui| {
            if tui.turbo_vision.quit_requested {
                None
            } else {
                tui.turbo_vision.pending_commands.pop_front()
            }
        });

        let Some(command) = command else {
            return Ok(ProcessOutcome::Idle);
        };

        Ok(self
            .dispatch_turbo_vision_command_event(&Event::command(command), line)?
            .unwrap_or(ProcessOutcome::Idle))
    }

    pub(super) fn turbo_vision_quit(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        self.pop_tui_application(line)?;
        self.with_tui(|tui| {
            tui.quit_requested = true;
            tui.turbo_vision.quit_requested = true;
        });
        Ok(())
    }

    pub(in crate::vm::execute::io) fn turbo_vision_application_run(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                "Application.Run(App) for Turbo Vision must run on the main task",
                "Call `Application.Run(App)` from the main program, not from a `go` task.",
                line,
            ));
        }

        if self.with_tui(|tui| tui.session.is_headless()) {
            return self.turbo_vision_headless_run(line);
        }

        let mut app = self.build_turbo_vision_application(line)?;
        app.run();
        Ok(())
    }

    fn turbo_vision_headless_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        for _ in 0..HEADLESS_RUN_MAX_COMMANDS {
            let stop = self.with_tui(|tui| {
                tui.turbo_vision.quit_requested
                    || (tui.turbo_vision.pending_commands.is_empty() && !tui.quit_requested)
            });
            if stop {
                return Ok(());
            }
            let _ = self.turbo_vision_pump_next_command(line)?;
        }

        Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            format!(
                "Application.Run(App) for Turbo Vision exceeded {HEADLESS_RUN_MAX_COMMANDS} queued command iterations"
            ),
            "Call `Application.Quit(App)` from the command handler or stop queueing commands.",
            line,
        ))
    }

    fn build_turbo_vision_application(
        &self,
        line: SourceLocation,
    ) -> Result<TurboVisionApplication, VmError> {
        let snapshots = self.turbo_vision_dialog_snapshots();
        let mut app = TurboVisionApplication::new().map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Turbo Vision terminal initialization failed: {error}"),
                "Run the program from an interactive terminal or use `Application.OpenForTest` in automated tests.",
                line,
            )
        })?;

        for dialog in snapshots {
            let mut dialog_view = Dialog::new_modal(turbo_rect(dialog.bounds), &dialog.title);
            for button in dialog.buttons {
                dialog_view.add(Box::new(Button::new(
                    turbo_rect(button.bounds),
                    &button.text,
                    button.command_id,
                    false,
                )));
            }
            app.desktop.add(dialog_view);
        }

        Ok(app)
    }

    fn turbo_vision_dialog_snapshots(&self) -> Vec<TurboVisionDialogSnapshot> {
        self.with_tui(|tui| {
            tui.turbo_vision
                .objects
                .values()
                .filter_map(|object| {
                    let TurboVisionObject::Dialog(dialog) = object else {
                        return None;
                    };
                    Some(TurboVisionDialogSnapshot {
                        bounds: dialog.bounds,
                        title: dialog.title.clone(),
                        buttons: dialog
                            .children
                            .iter()
                            .filter_map(|handle| match tui.turbo_vision.objects.get(handle) {
                                Some(TurboVisionObject::Button(button)) => Some(button.clone()),
                                _ => None,
                            })
                            .collect(),
                    })
                })
                .collect()
        })
    }
}

struct TurboVisionDialogSnapshot {
    bounds: TurboVisionRect,
    title: String,
    buttons: Vec<TurboVisionButton>,
}

fn turbo_rect(rect: TurboVisionRect) -> turbo_vision::core::geometry::Rect {
    turbo_vision::core::geometry::Rect::from_coords(rect.x, rect.y, rect.width, rect.height)
}

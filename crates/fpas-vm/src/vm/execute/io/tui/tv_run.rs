//! Turbo Vision `Application.Run` integration.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use super::tv_geometry::turbo_rect;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use crate::vm::shared::{TurboVisionButton, TurboVisionObject, TurboVisionRect};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::app::Application as TurboVisionApplication;
use turbo_vision::views::{button::Button, dialog::Dialog, window::Window};

const HEADLESS_RUN_MAX_COMMANDS: usize = 4096;

impl Worker {
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
        let window_snapshots = self.turbo_vision_window_snapshots();
        let dialog_snapshots = self.turbo_vision_dialog_snapshots();
        let mut app = TurboVisionApplication::new().map_err(|error| {
            runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!("Turbo Vision terminal initialization failed: {error}"),
                "Run the program from an interactive terminal or use `Application.OpenForTest` in automated tests.",
                line,
            )
        })?;

        for window in window_snapshots {
            let mut window_view = Window::new(turbo_rect(window.bounds), &window.title);
            for button in window.buttons {
                window_view.add(Box::new(Button::new(
                    turbo_rect(button.bounds),
                    &button.text,
                    button.command_id,
                    false,
                )));
            }
            app.desktop.add(Box::new(window_view));
        }

        for dialog in dialog_snapshots {
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

    fn turbo_vision_window_snapshots(&self) -> Vec<TurboVisionWindowSnapshot> {
        self.with_tui(|tui| {
            tui.turbo_vision
                .objects
                .values()
                .filter_map(|object| {
                    let TurboVisionObject::Window(window) = object else {
                        return None;
                    };
                    if !window.on_desktop {
                        return None;
                    }
                    Some(TurboVisionWindowSnapshot {
                        bounds: window.bounds,
                        title: window.title.clone(),
                        buttons: window
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

struct TurboVisionWindowSnapshot {
    bounds: TurboVisionRect,
    title: String,
    buttons: Vec<TurboVisionButton>,
}

struct TurboVisionDialogSnapshot {
    bounds: TurboVisionRect,
    title: String,
    buttons: Vec<TurboVisionButton>,
}

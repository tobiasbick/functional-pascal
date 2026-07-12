//! Turbo Vision bridge `Application.Run` (headless and live upstream event loops).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`

use super::app::bridge_ensure_live_app;
use super::chrome::bridge_sync_chrome_to_app;
use super::events::bridge_dispatch_on_command;
use super::headless::bridge_ensure_headless_app;
use super::headless_draw::HeadlessRunStep;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;

const TUI_HEADLESS_RUN_MAX_IDLE_STEPS: usize = 4096;

/// Runs the Turbo Vision event loop until `Application.Quit` or `CM_QUIT`.
pub(in crate::vm::execute::io::tui) fn bridge_application_run(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.current_task_id != 0 {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.Run(App) for Turbo Vision must run on the main task",
            "Call `Application.Run(App)` from the main program, not from a `go` task.",
            line,
        ));
    }

    worker.pop_tui_application(line)?;

    if worker.with_tui(|tui| tui.on_command.is_none()) {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.Run(App) requires a command handler",
            "Call `Application.Configure(App, Handlers)` with `OnCommand` set, register `Application.OnCommand`, or pass the handler to `Application.Run(App, OnCommand)`.",
            line,
        ));
    }

    bridge_application_run_loop(worker, line)
}

pub(in crate::vm::execute::io::tui) fn bridge_application_run_loop(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    bridge_sync_chrome_to_app(worker, line)?;

    if worker.with_tui(|tui| tui.session.is_headless()) {
        return bridge_headless_application_run(worker, line);
    }

    bridge_live_application_run(worker, line)
}

fn bridge_headless_application_run(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    bridge_ensure_headless_app(worker, line)?;
    let mut idle_steps = 0usize;

    loop {
        if worker.with_tui(|tui| tui.quit_requested) {
            break;
        }

        let step = {
            let Some(app) = worker.headless_tv_app.as_mut() else {
                return Err(runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    "Headless Turbo Vision session is not initialized",
                    "Call `Application.OpenForTest` before `Application.Run`.",
                    line,
                ));
            };
            app.run_step_outcome().map_err(|error| {
                runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Headless Turbo Vision run step failed: {error}"),
                    "Retry after `Application.OpenForTest` or reduce synthetic event volume.",
                    line,
                )
            })?
        };

        match step {
            HeadlessRunStep::Command(command) => {
                idle_steps = 0;
                bridge_dispatch_on_command(worker, command, line)?;
                if command == CM_QUIT || worker.with_tui(|tui| tui.quit_requested) {
                    break;
                }
            }
            HeadlessRunStep::Unhandled(mut event) => {
                idle_steps = 0;
                worker.dispatch_turbo_vision_unhandled_input(&mut event, line)?;
                if worker.with_tui(|tui| tui.quit_requested) {
                    break;
                }
            }
            HeadlessRunStep::Idle => {
                idle_steps = idle_steps.saturating_add(1);
                if idle_steps > TUI_HEADLESS_RUN_MAX_IDLE_STEPS {
                    return Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        format!(
                            "Application.Run(App) exceeded {TUI_HEADLESS_RUN_MAX_IDLE_STEPS} idle headless iterations"
                        ),
                        "Call `Application.Quit(App)` from the command handler or inject a command event.",
                        line,
                    ));
                }
            }
        }
    }

    worker.apply_pending_mouse_state_toggles(line)?;
    worker.turbo_vision_export_headless_to_console();
    Ok(())
}

fn bridge_live_application_run(worker: &mut Worker, line: SourceLocation) -> Result<(), VmError> {
    bridge_ensure_live_app(worker, line)?;
    worker.turbo_vision_set_live_app_running(true);

    loop {
        if !worker.turbo_vision_live_app_running() || worker.with_tui(|tui| tui.quit_requested) {
            return Ok(());
        }

        let Some(mut event) = worker.turbo_vision_live_next_event() else {
            continue;
        };

        worker.turbo_vision_live_handle_event(&mut event);

        if event.what == EventType::Command {
            bridge_dispatch_on_command(worker, event.command, line)?;
            if event.command == CM_QUIT || worker.with_tui(|tui| tui.quit_requested) {
                return Ok(());
            }
        } else if event.what != EventType::Nothing {
            worker.dispatch_turbo_vision_unhandled_input(&mut event, line)?;
        }

        worker.turbo_vision_live_after_step();
    }
}

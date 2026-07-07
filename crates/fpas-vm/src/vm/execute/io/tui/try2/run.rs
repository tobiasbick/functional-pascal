//! Try-2 `Application.Run` (headless and live upstream event loops).
//!
//! **Documentation:** `docs/refactor-tui-try-2/target-api.md`

use super::app::try2_ensure_live_app;
use super::events::try2_dispatch_on_command;
use super::headless::try2_ensure_headless_app;
use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;

const TRY2_HEADLESS_RUN_MAX_IDLE_STEPS: usize = 4096;

/// Runs the try-2 event loop until `Application.Quit` or `CM_QUIT`.
pub(in crate::vm::execute::io::tui) fn try2_application_run(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    if worker.current_task_id != 0 {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.Run(App) for try-2 must run on the main task",
            "Call `Application.Run(App)` from the main program, not from a `go` task.",
            line,
        ));
    }

    worker.pop_tui_application(line)?;

    if worker.with_tui(|tui| tui.on_command.is_none()) {
        return Err(runtime_error(
            RUNTIME_CONSOLE_STATE_ERROR,
            "Application.Run(App) requires `Application.OnCommand(App, Handler)` on the try-2 path",
            "Register a command handler before starting the run loop.",
            line,
        ));
    }

    if worker.with_tui(|tui| tui.session.is_headless()) {
        return try2_headless_application_run(worker, line);
    }

    try2_live_application_run(worker, line)
}

fn try2_headless_application_run(
    worker: &mut Worker,
    line: SourceLocation,
) -> Result<(), VmError> {
    try2_ensure_headless_app(worker, line)?;
    let mut idle_steps = 0usize;

    loop {
        if worker.with_tui(|tui| tui.quit_requested) {
            break;
        }

        let command = {
            let Some(app) = worker.headless_tv_app.as_mut() else {
                return Err(runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    "Headless Turbo Vision session is not initialized",
                    "Call `Application.OpenForTest` before `Application.Run`.",
                    line,
                ));
            };
            app.run_step().map_err(|error| {
                runtime_error(
                    RUNTIME_CONSOLE_STATE_ERROR,
                    format!("Headless Turbo Vision run step failed: {error}"),
                    "Retry after `Application.OpenForTest` or reduce synthetic event volume.",
                    line,
                )
            })?
        };

        match command {
            Some(command) => {
                idle_steps = 0;
                try2_dispatch_on_command(worker, command, line)?;
                if command == CM_QUIT || worker.with_tui(|tui| tui.quit_requested) {
                    break;
                }
            }
            None => {
                idle_steps = idle_steps.saturating_add(1);
                if idle_steps > TRY2_HEADLESS_RUN_MAX_IDLE_STEPS {
                    return Err(runtime_error(
                        RUNTIME_CONSOLE_STATE_ERROR,
                        format!(
                            "Application.Run(App) exceeded {TRY2_HEADLESS_RUN_MAX_IDLE_STEPS} idle headless iterations"
                        ),
                        "Call `Application.Quit(App)` from the command handler or inject a command event.",
                        line,
                    ));
                }
            }
        }
    }

    worker.turbo_vision_export_headless_to_console();
    Ok(())
}

fn try2_live_application_run(worker: &mut Worker, line: SourceLocation) -> Result<(), VmError> {
    try2_ensure_live_app(worker, line)?;
    worker.turbo_vision_set_live_app_running(true);

    loop {
        if !worker.turbo_vision_live_app_running()
            || worker.with_tui(|tui| tui.quit_requested)
        {
            return Ok(());
        }

        let Some(mut event) = worker.turbo_vision_live_next_event() else {
            continue;
        };

        worker.turbo_vision_live_handle_event(&mut event);

        if event.what == EventType::Command {
            try2_dispatch_on_command(worker, event.command, line)?;
            if event.command == CM_QUIT || worker.with_tui(|tui| tui.quit_requested) {
                return Ok(());
            }
        } else if event.what != EventType::Nothing {
            worker.dispatch_turbo_vision_unhandled_input(&mut event, line)?;
        }

        worker.turbo_vision_live_after_step();
    }
}

//! Hosted `Std.Tui.Application.Run` execution.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

const TUI_EXIT_REASON_TYPE: &str = "Std.Tui.ExitReason";
const USER_QUIT_EXIT_REASON: &str = "UserQuit";
const HOST_STOP_EXIT_REASON: &str = "HostStop";
const DEFAULT_RUN_WAIT_TIMEOUT_MS: i64 = 50;
const RUN_PROCESS_SPINS: usize = 64;

enum IdleWaitOutcome {
    Continue,
    InvokeOnIdle,
}

impl Worker {
    pub(super) fn tui_application_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.pop_tui_application(line)?;
        self.prepare_tui_application_run(line)?;

        let run_result = self.tui_application_run_loop(line);
        let close_result = self.close_tui_application_state(line);

        match (run_result, close_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    fn prepare_tui_application_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) must run on the main task",
                "Call `Application.Run(App)` from the main program, not from a `go` task.",
                line,
            ));
        }

        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.run_active {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) cannot start while another hosted TUI run is active",
                "Return from the current `On*` handler before starting another hosted run.",
                line,
            ));
        }
        if tui.on_paint.is_none() {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) requires a registered OnPaint handler",
                "Call `Application.HostRegisterOnPaint(App, OnPaint)` before `Application.Run(App)`.",
                line,
            ));
        }
        tui.run_active = true;
        Ok(())
    }

    fn tui_application_run_loop(&mut self, line: SourceLocation) -> Result<(), VmError> {
        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.session.request_redraw(line)?;
        }

        loop {
            let redraw_tag = self.tui_host_dispatch_redraw_inner(line)?;
            let process_tag = self.tui_host_process_next_inner(RUN_PROCESS_SPINS, line)?;

            if matches!(process_tag, 2 | 4) {
                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.session.request_redraw(line)?;
            }

            if let Some(exit_reason) = self.take_tui_application_run_stop_reason() {
                return self.finish_tui_application_run(exit_reason, line);
            }

            if redraw_tag != 0 || process_tag != 0 {
                continue;
            }

            match self.wait_for_tui_run_work_or_idle(line)? {
                IdleWaitOutcome::Continue => {}
                IdleWaitOutcome::InvokeOnIdle => {
                    self.invoke_tui_on_idle_if_present(line)?;
                    if let Some(exit_reason) = self.take_tui_application_run_stop_reason() {
                        return self.finish_tui_application_run(exit_reason, line);
                    }
                }
            }
        }
    }

    fn take_tui_application_run_stop_reason(&self) -> Option<Value> {
        let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
        if tui.host_stop_requested {
            tui.host_stop_requested = false;
            Some(Self::host_stop_exit_reason())
        } else if tui.quit_requested {
            tui.quit_requested = false;
            Some(Self::user_quit_exit_reason())
        } else {
            None
        }
    }

    fn wait_for_tui_run_work_or_idle(
        &mut self,
        line: SourceLocation,
    ) -> Result<IdleWaitOutcome, VmError> {
        let (idle_enabled, wait_timeout_ms) = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let idle_enabled = tui.idle_interval_ms > 0 && tui.on_idle.is_some();
            let wait_timeout_ms = if idle_enabled {
                tui.idle_interval_ms
            } else {
                DEFAULT_RUN_WAIT_TIMEOUT_MS
            };
            (idle_enabled, wait_timeout_ms)
        };

        let flushed_resize = {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            let next = self.with_console_and_key_input(|console, key_input| {
                tui.session
                    .read_event_timeout(console, key_input, wait_timeout_ms, line)
            })?;
            match next {
                Some(event) => {
                    tui.host.ingest_tui_event(event);
                    false
                }
                None => tui.host.flush_pending_resize(),
            }
        };

        if flushed_resize {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.session.request_redraw(line)?;
            return Ok(IdleWaitOutcome::Continue);
        }

        if idle_enabled {
            return Ok(IdleWaitOutcome::InvokeOnIdle);
        }

        Ok(IdleWaitOutcome::Continue)
    }

    fn finish_tui_application_run(
        &mut self,
        exit_reason: Value,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        {
            let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.last_exit_reason = Some(exit_reason.clone());
        }
        self.invoke_tui_on_exit_if_present(exit_reason, line)
    }

    fn invoke_tui_on_idle_if_present(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_idle.clone()
        };

        let Some(handler) = handler else {
            return Ok(());
        };

        let _ = self.call_function_sync(&handler, &[Self::tui_application_record()], line)?;
        Ok(())
    }

    fn invoke_tui_on_exit_if_present(
        &mut self,
        exit_reason: Value,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.on_exit.clone()
        };

        let Some(handler) = handler else {
            return Ok(());
        };

        let _ = self.call_function_sync(
            &handler,
            &[Self::tui_application_record(), exit_reason],
            line,
        )?;
        Ok(())
    }

    fn user_quit_exit_reason() -> Value {
        Value::Enum {
            type_name: TUI_EXIT_REASON_TYPE.into(),
            variant: USER_QUIT_EXIT_REASON.into(),
            fields: vec![],
        }
    }

    fn host_stop_exit_reason() -> Value {
        Value::Enum {
            type_name: TUI_EXIT_REASON_TYPE.into(),
            variant: HOST_STOP_EXIT_REASON.into(),
            fields: vec![],
        }
    }
}

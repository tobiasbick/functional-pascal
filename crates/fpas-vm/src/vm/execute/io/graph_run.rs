//! Hosted `Std.Graph.Application.Run` execution.
//!
//! **Documentation:** `docs/pascal/std/graph/app/README.md` (from the repository root).

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::UiEvent;

use super::hosted_common::hosted_exit_reason;

const GRAPH_EXIT_REASON_TYPE: &str = "Std.Graph.ExitReason";
const USER_QUIT_EXIT_REASON: &str = "UserQuit";
const WINDOW_CLOSED_EXIT_REASON: &str = "WindowClosed";
const HOST_STOP_EXIT_REASON: &str = "HostStop";
const HOST_AND_USER_STOP_EXIT_REASON: &str = "HostAndUserStop";
const HOST_SHUTDOWN_EXIT_REASON: &str = "HostShutdown";
const DEFAULT_RUN_WAIT_TIMEOUT_MS: i64 = 50;
const RUN_PROCESS_SPINS: usize = 64;

enum IdleWaitOutcome {
    Continue,
    InvokeOnIdle,
}

impl Worker {
    pub(super) fn graph_application_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.pop_graph_application(line)?;
        self.prepare_graph_application_run(line)?;

        let run_result = self.graph_application_run_loop(line);
        let close_result = self.close_graph_application_state(line);

        match (run_result, close_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    fn prepare_graph_application_run(&mut self, line: SourceLocation) -> Result<(), VmError> {
        if self.current_task_id != 0 {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) must run on the main task",
                "Call `Application.Run(App)` from the main program, not from a `go` task.",
                line,
            ));
        }

        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        if graph.run_active {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) cannot start while another hosted graph run is active",
                "Return from the current `On*` handler before starting another hosted run.",
                line,
            ));
        }
        if graph.on_paint.is_none() {
            return Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Application.Run(App) requires a registered OnPaint handler",
                "Call `Application.Configure(App, Handlers)` with `OnPaint` before `Application.Run(App)`.",
                line,
            ));
        }
        graph.run_active = true;
        Ok(())
    }

    fn graph_application_run_loop(&mut self, line: SourceLocation) -> Result<(), VmError> {
        self.prime_initial_graph_run_events(line)?;
        self.dispatch_initial_graph_resize_if_present(line)?;
        self.with_graph(|graph| graph.session.request_redraw_if_absent(line))?;

        loop {
            let redraw_tag = self.graph_host_dispatch_redraw_inner(line)?;
            let process_tag = self.graph_host_process_next_inner(RUN_PROCESS_SPINS, line)?;

            if let Some(exit_reason) = self.take_graph_application_run_stop_reason() {
                return self.finish_graph_application_run(exit_reason, line);
            }

            if redraw_tag != 0 || process_tag != 0 {
                continue;
            }

            match self.wait_for_graph_run_work_or_idle(line)? {
                IdleWaitOutcome::Continue => {}
                IdleWaitOutcome::InvokeOnIdle => {
                    self.invoke_graph_on_idle_if_present(line)?;
                    if let Some(exit_reason) = self.take_graph_application_run_stop_reason() {
                        return self.finish_graph_application_run(exit_reason, line);
                    }
                }
            }
        }
    }

    fn prime_initial_graph_run_events(&mut self, line: SourceLocation) -> Result<(), VmError> {
        for _ in 0..RUN_PROCESS_SPINS {
            let next = {
                let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                graph.session.read_host_ui_event_timeout(0, line)?
            };

            let Some(event) = next else {
                break;
            };

            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            graph.host.ingest_ui_event(event);
        }

        Ok(())
    }

    fn dispatch_initial_graph_resize_if_present(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let has_initial_resize = {
            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            let _ = graph.host.flush_pending_resize();
            matches!(graph.host.peek_ready_event(), Some(UiEvent::Resize(_)))
        };

        if has_initial_resize {
            let _ = self.graph_host_process_next_inner(1, line)?;
        }

        Ok(())
    }

    fn take_graph_application_run_stop_reason(&self) -> Option<Value> {
        if self.shared.is_shutdown() {
            return Some(Self::graph_host_shutdown_exit_reason());
        }

        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        if graph.window_closed {
            graph.window_closed = false;
            return Some(Self::graph_window_closed_exit_reason());
        }
        if graph.host_stop_requested {
            let quit_requested = graph.quit_requested;
            graph.host_stop_requested = false;
            graph.quit_requested = false;
            if quit_requested {
                Some(Self::graph_host_and_user_stop_exit_reason())
            } else {
                Some(Self::graph_host_stop_exit_reason())
            }
        } else if graph.quit_requested {
            graph.quit_requested = false;
            Some(Self::graph_user_quit_exit_reason())
        } else {
            None
        }
    }

    fn wait_for_graph_run_work_or_idle(
        &mut self,
        line: SourceLocation,
    ) -> Result<IdleWaitOutcome, VmError> {
        let (idle_enabled, wait_timeout_ms) = self.with_graph(|graph| {
            let idle_enabled = graph.idle_interval_ms > 0 && graph.on_idle.is_some();
            let wait_timeout_ms = if idle_enabled {
                graph.idle_interval_ms
            } else {
                DEFAULT_RUN_WAIT_TIMEOUT_MS
            };
            (idle_enabled, wait_timeout_ms)
        });

        let flushed_resize = {
            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            let next = graph
                .session
                .read_host_ui_event_timeout(wait_timeout_ms, line)?;
            match next {
                Some(event) => {
                    graph.host.ingest_ui_event(event);
                    false
                }
                None => graph.host.flush_pending_resize(),
            }
        };

        if flushed_resize {
            return Ok(IdleWaitOutcome::Continue);
        }

        if idle_enabled {
            return Ok(IdleWaitOutcome::InvokeOnIdle);
        }

        Ok(IdleWaitOutcome::Continue)
    }

    fn finish_graph_application_run(
        &mut self,
        exit_reason: Value,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.with_graph(|graph| {
            graph.last_exit_reason = Some(exit_reason.clone());
        });
        self.invoke_graph_on_exit_if_present(exit_reason, line)
    }

    fn invoke_graph_on_idle_if_present(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let handler = self.with_graph(|graph| graph.on_idle.clone());
        let Some(handler) = handler else {
            return Ok(());
        };
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[Self::graph_application_record()],
            line,
        )?;
        Ok(())
    }

    fn invoke_graph_on_exit_if_present(
        &mut self,
        exit_reason: Value,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handler = self.with_graph(|graph| graph.on_exit.clone());
        let Some(handler) = handler else {
            return Ok(());
        };

        let previous_allow_shutdown = self.allow_shutdown_during_sync_call;
        self.allow_shutdown_during_sync_call = true;
        let callback_result = self.call_function_sync(
            &handler,
            &[Self::graph_application_record(), exit_reason],
            line,
        );
        self.allow_shutdown_during_sync_call = previous_allow_shutdown;
        let _ = callback_result?;
        Ok(())
    }

    fn graph_user_quit_exit_reason() -> Value {
        hosted_exit_reason(GRAPH_EXIT_REASON_TYPE, USER_QUIT_EXIT_REASON)
    }

    fn graph_window_closed_exit_reason() -> Value {
        hosted_exit_reason(GRAPH_EXIT_REASON_TYPE, WINDOW_CLOSED_EXIT_REASON)
    }

    fn graph_host_stop_exit_reason() -> Value {
        hosted_exit_reason(GRAPH_EXIT_REASON_TYPE, HOST_STOP_EXIT_REASON)
    }

    fn graph_host_and_user_stop_exit_reason() -> Value {
        hosted_exit_reason(GRAPH_EXIT_REASON_TYPE, HOST_AND_USER_STOP_EXIT_REASON)
    }

    fn graph_host_shutdown_exit_reason() -> Value {
        hosted_exit_reason(GRAPH_EXIT_REASON_TYPE, HOST_SHUTDOWN_EXIT_REASON)
    }
}

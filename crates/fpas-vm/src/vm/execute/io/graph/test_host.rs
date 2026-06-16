//! Headless native graph testing intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph-app.md`, `docs/pascal/std/test.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR;
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{GraphEvent, HeadlessGraphTestModeGuard};

impl Worker {
    /// Executes headless native graph testing intrinsics.
    pub(super) fn try_exec_graph_test_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Graph(GraphIntrinsic::OpenForTest) => {
                let height = self.pop_int(line)?;
                let width = self.pop_int(line)?;
                Self::validate_graph_test_dimension(width, "Width", line)?;
                Self::validate_graph_test_dimension(height, "Height", line)?;
                if self.current_task_id != 0 {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Std.Graph.Application.* must run on the main task",
                        "Call `Std.Graph.Application.*` from the main program, not from a `go` task.",
                        line,
                    ));
                }

                let headless_guard = HeadlessGraphTestModeGuard::push();
                let open_result = (|| -> Result<(), VmError> {
                    let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    if graph.session.is_open() {
                        return Err(open_for_test_second_session_error(line));
                    }
                    graph.session.open(width, height, "", line)?;
                    graph.headless_test_open = true;
                    let pending = std::mem::take(&mut graph.pending_test_events);
                    for event in pending {
                        graph.session.push_event(event, line)?;
                    }
                    Ok(())
                })();

                match open_result {
                    Ok(()) => {
                        headless_guard.release();
                        self.push(Self::graph_application_record())?;
                    }
                    Err(error) => return Err(error),
                }
            }
            Intrinsic::Graph(GraphIntrinsic::TestSendKey) => {
                let key = self.pop_console_key_event(line)?;
                self.pop_graph_application(line)?;
                self.enqueue_graph_test_event(GraphEvent::Key(key), line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn validate_graph_test_dimension(
        value: i64,
        name: &str,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if value <= 0 {
            return Err(runtime_error(
                RUNTIME_CONSOLE_STATE_ERROR,
                format!(
                    "Application.OpenForTest({name}, …) requires positive {name}, got {value}."
                ),
                "Pass positive pixel dimensions, e.g. `Application.OpenForTest(640, 480)`.",
                line,
            ));
        }
        Ok(())
    }

    fn enqueue_graph_test_event(
        &mut self,
        event: GraphEvent,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        if graph.session.is_open() {
            graph.session.push_event(event, line)?;
        } else {
            graph.pending_test_events.push(event);
        }
        Ok(())
    }
}

fn open_for_test_second_session_error(line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Application.OpenForTest(Width, Height) cannot open a second graphics session while one is already active.",
        "Close the current graphics session with `Application.Close(App)` before opening another one.",
        line,
    )
}

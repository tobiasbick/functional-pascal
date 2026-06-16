//! Headless native graph testing intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph-app.md`, `docs/pascal/std/test.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{GraphIntrinsic, Intrinsic, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::{GraphEvent, push_headless_graph_test_mode};

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
                if self.current_task_id != 0 {
                    return Err(runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "Std.Graph.Application.* must run on the main task",
                        "Call `Std.Graph.Application.*` from the main program, not from a `go` task.",
                        line,
                    ));
                }
                push_headless_graph_test_mode();
                {
                    let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
                    if graph.session.is_open() {
                        return Err(open_for_test_second_session_error(line));
                    }
                    graph.headless_test_open = true;
                    graph.session.open(width, height, "", line)?;
                    let pending = std::mem::take(&mut graph.pending_test_events);
                    for event in pending {
                        graph.session.push_event(event, line)?;
                    }
                }
                self.push(Self::graph_application_record())?;
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

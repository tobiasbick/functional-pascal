//! Hosted `Std.Graph` lifecycle helpers.
//!
//! **Documentation:** `docs/pascal/std/graph/app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;

impl Worker {
    pub(in crate::vm::execute::io) fn request_graph_host_stop_for_active_run(&self) -> bool {
        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        if graph.run_active {
            graph.host_stop_requested = true;
            true
        } else {
            false
        }
    }

    /// Closes the graph session and resets hosted-dispatch state.
    pub(in crate::vm::execute::io) fn close_graph_application_state(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
        let close_result = graph.session.close(line);
        graph.host = fpas_std::UiHost::for_graph();
        graph.on_key_pressed = None;
        graph.on_mouse = None;
        graph.on_wheel = None;
        graph.on_resize = None;
        graph.on_close_requested = None;
        graph.on_paint = None;
        graph.on_idle = None;
        graph.idle_interval_ms = 0;
        graph.on_exit = None;
        graph.last_exit_reason = None;
        graph.quit_requested = false;
        graph.window_closed = false;
        graph.host_stop_requested = false;
        graph.run_active = false;
        graph.pending_test_events.clear();
        if graph.headless_test_open {
            fpas_std::pop_headless_graph_test_mode();
            graph.headless_test_open = false;
        }
        close_result?;
        Ok(())
    }
}

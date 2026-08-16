//! Session-owned TUI and graph event queues distinct from protocol bytes.

#[cfg(test)]
use super::*;
#[cfg(test)]
use crate::vm::GraphState;
#[cfg(test)]
use fpas_std::{ConsoleEvent, GraphEvent};

#[cfg(test)]
impl DebugSession {
    /// Queue one TUI event for hosted `PollEvent` / `ReadEvent` without OS polling.
    pub(in crate::vm::debug) fn test_push_console_event(&self, event: ConsoleEvent) {
        self.with_key_input(|input| input.push_console_event(event));
    }

    /// Queue one graph event until `Application.Run` dispatches it as bytecode.
    pub(in crate::vm::debug) fn test_push_graph_event(&self, event: GraphEvent) {
        self.with_graph(|graph| {
            if graph
                .session
                .push_event(event.clone(), fpas_bytecode::SourceLocation::new(1, 1))
                .is_err()
            {
                graph.pending_test_events.push(event);
            }
        });
    }

    fn with_graph<R>(&self, operation: impl FnOnce(&mut GraphState) -> R) -> R {
        let Some(worker) = self.runtime.worker(0) else {
            unreachable!("debug runtime always retains the main task")
        };
        let mut graph = worker
            .hosted
            .graph
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut graph)
    }
}

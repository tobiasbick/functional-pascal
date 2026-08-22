//! Session-owned TUI event queues distinct from protocol bytes.

#[cfg(test)]
use super::*;
#[cfg(test)]
use fpas_std::ConsoleEvent;

#[cfg(test)]
impl DebugSession {
    /// Queue one TUI event for hosted `PollEvent` / `ReadEvent` without OS polling.
    pub(in crate::vm::debug) fn test_push_console_event(&self, event: ConsoleEvent) {
        self.with_key_input(|input| input.push_console_event(event));
    }
}

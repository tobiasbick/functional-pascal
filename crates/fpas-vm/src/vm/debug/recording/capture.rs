//! In-memory capture of all-stop and queued debuggee-input events.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::types::{DebugStop, DebugStopReason};

/// Largest number of captured recording events retained in memory.
pub const MAX_RECORDING_EVENTS: usize = 4_096;

/// One captured scheduler or host-input observation.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugRecordingEvent {
    /// An all-stop boundary with the responsible task.
    Stop {
        /// Runtime task identity responsible for the stop.
        task_id: u64,
        /// Why the session stopped.
        reason: DebugStopReason,
        /// Bytecode instruction address at the stop.
        instruction: u32,
    },
    /// One line queued for hosted `Read` / `ReadLn` while capturing.
    Input {
        /// Exact queued text, without the stored newline.
        text: String,
    },
}

/// Bounded capture log. Recording is off until [`DebugRecordingLog::start`].
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DebugRecordingLog {
    capturing: bool,
    truncated: bool,
    events: Vec<DebugRecordingEvent>,
}

impl DebugRecordingLog {
    /// Whether later stops and queued input are appended.
    #[must_use]
    pub const fn capturing(&self) -> bool {
        self.capturing
    }

    /// Whether later events were dropped after the in-memory ceiling.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Captured events in order. Empty while recording is off.
    #[must_use]
    pub fn events(&self) -> &[DebugRecordingEvent] {
        &self.events
    }

    /// Enable capture. Returns whether this call started a new log.
    pub fn start(&mut self) -> bool {
        if self.capturing {
            return false;
        }
        self.capturing = true;
        true
    }

    /// Append one all-stop when capturing and below the event ceiling.
    pub fn push_stop(&mut self, stop: &DebugStop) {
        self.push(DebugRecordingEvent::Stop {
            task_id: stop.task_id,
            reason: stop.reason,
            instruction: stop.instruction,
        });
    }

    /// Append one queued debuggee line when capturing and below the event ceiling.
    pub fn push_input(&mut self, text: &str) {
        self.push(DebugRecordingEvent::Input {
            text: text.to_owned(),
        });
    }

    fn push(&mut self, event: DebugRecordingEvent) {
        if !self.capturing {
            return;
        }
        if self.events.len() >= MAX_RECORDING_EVENTS {
            self.truncated = true;
            return;
        }
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::debug::types::DebugStopReason;

    fn stop(task_id: u64) -> DebugStop {
        DebugStop {
            reason: DebugStopReason::Entry,
            task_id,
            location: None,
            instruction: 0,
            call_depth: 0,
            breakpoint_id: None,
            breakpoint_ids: Vec::new(),
            diagnostic: None,
        }
    }

    #[test]
    fn log_stays_empty_until_start() {
        let mut log = DebugRecordingLog::default();
        log.push_stop(&stop(0));
        log.push_input("queued");
        assert!(!log.capturing());
        assert!(!log.truncated());
        assert!(log.events().is_empty());
    }

    #[test]
    fn start_is_idempotent_and_respects_the_event_ceiling() {
        let mut log = DebugRecordingLog::default();
        assert!(log.start());
        assert!(!log.start());
        for index in 0..MAX_RECORDING_EVENTS + 8 {
            log.push_stop(&stop(index as u64));
        }
        assert_eq!(log.events().len(), MAX_RECORDING_EVENTS);
        assert!(log.capturing());
        assert!(log.truncated());
    }
}

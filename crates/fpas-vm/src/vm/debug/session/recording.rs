//! Session mapping onto the recording envelope and capture log.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use crate::vm::debug::recording::{DebugRecordingEnvelope, DebugRecordingEvent};

impl DebugSession {
    /// Return the versioned program identity for this session.
    ///
    /// The envelope names portable sources and the entry function. It does not
    /// start recording, resume, or mutate the debuggee.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a host-path error when a source identity is not portable.
    pub fn recording_envelope(&self) -> Result<DebugRecordingEnvelope, DebugSessionError> {
        DebugRecordingEnvelope::from_executable(&self.executable)
    }

    /// Start capturing all-stop and queued `Read`/`ReadLn` events.
    ///
    /// Recording is off until this is called. The current stop is recorded once
    /// when capture starts. Reverse execution stays unsupported. Later resume
    /// rejects unsupported host effects with `F4024` before they run.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    pub fn start_recording(&mut self) {
        if !self.recording.start() {
            return;
        }
        if matches!(
            self.state,
            DebugSessionState::Stopped | DebugSessionState::Failed
        ) {
            self.recording.push_stop(&self.last_stop);
        }
    }

    /// Whether all-stop and queued input events are being captured.
    #[must_use]
    pub const fn is_recording(&self) -> bool {
        self.recording.capturing()
    }

    /// Captured recording events in order.
    #[must_use]
    pub fn recording_events(&self) -> &[DebugRecordingEvent] {
        self.recording.events()
    }

    /// Whether later capture events were dropped after the in-memory ceiling.
    #[must_use]
    pub const fn recording_truncated(&self) -> bool {
        self.recording.truncated()
    }

    pub(super) fn capture_current_stop(&mut self) {
        self.recording.push_stop(&self.last_stop);
    }

    pub(super) fn capture_input(&mut self, text: &str) {
        self.recording.push_input(text);
    }

    /// Reject the pending instruction when capture cannot record that host effect.
    ///
    /// Recording-off execution skips this check. A diagnostic points at the
    /// pending intrinsic and leaves it unexecuted.
    pub(super) fn reject_unsupported_recording_effect(
        &mut self,
        task_id: u64,
    ) -> Option<fpas_diagnostics::Diagnostic> {
        if !self.recording.capturing() {
            return None;
        }
        let worker = self.runtime.worker_mut(task_id)?;
        let (address, diagnostic) =
            crate::vm::debug::recording::pending_unsupported_recording_effect(
                &worker.executable,
                worker.ip,
            )?;
        worker.current_address = address;
        Some(diagnostic)
    }
}

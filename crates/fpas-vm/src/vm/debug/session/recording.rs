//! Session mapping onto the recording envelope.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use crate::vm::debug::recording::DebugRecordingEnvelope;

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
}

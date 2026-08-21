//! Queued debuggee input, EOF, and cancel.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn push_debuggee_input(
        &mut self,
        request_id: u64,
        command: &str,
        text: String,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.push_debuggee_input(&text) {
            Ok(result) => vec![ok(
                request_id,
                command,
                ResponseBody::InputQueued {
                    bytes: result.bytes,
                    session_bytes: result.session_bytes,
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn signal_debuggee_eof(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.signal_debuggee_eof() {
            Ok(()) => vec![ok(request_id, command, ResponseBody::Eof)],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn cancel_debuggee_input(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.cancel_debuggee_input() {
            Ok(()) => vec![ok(request_id, command, ResponseBody::Cleared)],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

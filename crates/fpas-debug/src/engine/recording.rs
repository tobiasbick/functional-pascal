//! Recording envelope and capture log.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn describe_recording(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.recording_envelope() {
            Ok(envelope) => vec![ok(
                request_id,
                command,
                ResponseBody::Recording {
                    envelope,
                    capturing: session.is_recording(),
                    events: session.recording_events().to_vec(),
                    truncated: session.recording_truncated(),
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn start_recording(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        session.start_recording();
        vec![ok(
            request_id,
            command,
            ResponseBody::RecordingStarted {
                capturing: true,
                truncated: session.recording_truncated(),
                event_count: session.recording_events().len(),
            },
        )]
    }
}

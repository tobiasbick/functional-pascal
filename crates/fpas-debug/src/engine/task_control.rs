//! Per-task pause and resume holds.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error, task_event};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn pause_task(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: u64,
    ) -> Vec<DebugRecord> {
        self.set_task_hold(request_id, command, task_id, true)
    }

    pub(super) fn resume_task(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: u64,
    ) -> Vec<DebugRecord> {
        self.set_task_hold(request_id, command, task_id, false)
    }

    fn set_task_hold(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: u64,
        paused: bool,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let result = if paused {
            session.pause_task(task_id)
        } else {
            session.resume_task(task_id)
        };
        match result {
            Ok(()) => vec![ok(
                request_id,
                command,
                ResponseBody::TaskHold { task_id, paused },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn cancel_task(
        &mut self,
        request_id: u64,
        command: &str,
        task_id: u64,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.cancel_task(task_id) {
            Ok(()) => {
                let mut records = vec![ok(
                    request_id,
                    command,
                    ResponseBody::TaskCancelled { task_id },
                )];
                records.extend(session.take_task_events().into_iter().map(task_event));
                records
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

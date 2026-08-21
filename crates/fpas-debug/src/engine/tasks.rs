//! Task catalog requests.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn tasks(
        &mut self,
        request_id: u64,
        command: &str,
        start: usize,
        count: usize,
    ) -> Vec<DebugRecord> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.tasks(start, count) {
            Ok(tasks) => vec![ok(
                request_id,
                command,
                ResponseBody::Tasks {
                    tasks: tasks.items,
                    total: tasks.total,
                },
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

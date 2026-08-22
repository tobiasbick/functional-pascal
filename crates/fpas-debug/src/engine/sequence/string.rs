//! String character replacement.

use super::*;
use crate::engine::DebugEngine;
use crate::engine::record::{DebugRecord, ResponseBody};
use crate::engine::reply::{invalid_state, ok, session_error};

impl DebugEngine {
    pub(in crate::engine) fn replace_string_character(
        &mut self,
        request_id: u64,
        command: &str,
        target: String,
        index: String,
        expression: String,
        frame_id: Option<u64>,
    ) -> Vec<DebugRecord> {
        let request = match parse_request(
            request_id,
            command,
            self.status,
            &target,
            &[index, expression],
            frame_id,
        ) {
            Ok(request) => request,
            Err(response) => return vec![*response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.replace_string_character_with_limits(
            &request.target,
            &request.expressions[0],
            &request.expressions[1],
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![ok(
                request_id,
                command,
                ResponseBody::StringCharacter(result),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

//! Runtime-failure stop-filter configuration.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_request, invalid_state, ok};
use super::{DebugEngine, DebugStatus};
use crate::breakpoints::RuntimeFailurePolicy;

impl DebugEngine {
    pub(super) fn replace_runtime_failure_filters(
        &mut self,
        request_id: u64,
        command: &str,
        filters: Vec<String>,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let policy = match RuntimeFailurePolicy::parse(&filters) {
            Ok(policy) => policy,
            Err(error) => {
                return vec![invalid_request(
                    request_id,
                    command,
                    error.message,
                    error.hint,
                )];
            }
        };
        self.runtime_failure_policy = policy;
        vec![ok(
            request_id,
            command,
            ResponseBody::RuntimeFilters { filters },
        )]
    }
}

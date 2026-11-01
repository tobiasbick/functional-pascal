//! Bounded session-local recipes; handles never authorize edits without fresh validation.

use crate::documents::{
    DocumentRequestError,
    read_set::{Versions, failure},
};
use fpas_language_service::RefactorPlan;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct Saved {
    pub plan: RefactorPlan,
    pub versions: Versions,
}

pub(crate) struct Handles {
    session: String,
    next: u64,
    bytes: usize,
    entries: VecDeque<(String, usize, Arc<Saved>)>,
}

impl Default for Handles {
    fn default() -> Self {
        Self {
            session: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            next: 0,
            bytes: 0,
            entries: VecDeque::new(),
        }
    }
}

impl Handles {
    pub(crate) fn insert(&mut self, saved: Saved) -> Result<String, DocumentRequestError> {
        let bytes = serde_json::to_vec(&saved.plan)
            .map_err(|e| failure("invalid_plan", &e.to_string()))?
            .len();
        if bytes > fpas_language_service::REFACTOR_JSON_BYTES {
            return Err(failure(
                "plan_limit",
                "Refactoring plan exceeds the editor handle limit.",
            ));
        }
        while self.entries.len() >= 128 || self.bytes + bytes > 32 * 1024 * 1024 {
            if let Some((_, bytes, _)) = self.entries.pop_front() {
                self.bytes -= bytes;
            }
        }
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| failure("plan_limit", "Refactoring handle sequence exhausted."))?;
        let handle = format!("{}:{}", self.session, self.next);
        self.bytes += bytes;
        self.entries
            .push_back((handle.clone(), bytes, Arc::new(saved)));
        Ok(handle)
    }

    pub(crate) fn get(&self, handle: &str) -> Result<Arc<Saved>, DocumentRequestError> {
        self.entries
            .iter()
            .find(|(key, _, _)| key == handle)
            .map(|(_, _, value)| Arc::clone(value))
            .ok_or_else(|| {
                failure(
                    "stale_plan",
                    "Unknown or expired refactoring handle; prepare the recipe again.",
                )
            })
    }
}

//! Aliased one-use response rights, including abandonment and cancellation.

use super::state::Shared;
use crate::{
    Fault,
    model::{CallToolResult, ErrorData},
};
use std::sync::Arc;

struct Right {
    shared: Arc<Shared>,
    id: u64,
}

/// Aliases refer to one terminal response right; successful publication consumes it.
#[derive(Clone)]
pub struct Responder {
    right: Arc<Right>,
}

impl Responder {
    pub(super) fn new(shared: Arc<Shared>, id: u64) -> Self {
        Self {
            right: Arc::new(Right { shared, id }),
        }
    }

    /// Observe cancellation or any terminal response without granting effect authority.
    pub fn is_cancelled(&self) -> bool {
        self.right
            .shared
            .lock()
            .pending
            .get(&self.right.id)
            .is_none_or(|pending| pending.cancellation.is_cancelled() || pending.sender.is_closed())
    }

    /// Await cancellation or a terminal reply without consuming application events.
    pub async fn cancelled(&self) {
        loop {
            let changed = self.right.shared.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.is_cancelled() {
                return;
            }
            changed.await;
        }
    }

    /// Attachment owning this right; terminal rights return `Closed`.
    pub fn endpoint(&self) -> Result<u64, Fault> {
        self.right
            .shared
            .lock()
            .pending
            .get(&self.right.id)
            .map(|pending| pending.endpoint)
            .ok_or(Fault::Closed)
    }

    /// Validate and publish one result. Invalid output retains the unconsumed right.
    /// A tool error is a successful MCP response whose `is_error` field is true.
    pub fn respond(&self, result: Result<CallToolResult, ErrorData>) -> Result<(), Fault> {
        let shared = &self.right.shared;
        let mut state = shared.lock();
        let pending = state.pending.get(&self.right.id).ok_or(Fault::Closed)?;
        if pending.cancellation.is_cancelled() || pending.sender.is_closed() {
            state.discard(self.right.id);
            return Err(Fault::Closed);
        }
        let bytes = match &result {
            Ok(value) => {
                shared.catalog.validate_result(&pending.name, value)?;
                serde_json::to_vec(value)
            }
            Err(value) => serde_json::to_vec(value),
        }
        .map_err(|_| Fault::InvalidResult)?
        .len();
        if bytes > shared.catalog.limits.message_bytes {
            return Err(Fault::LimitExceeded(
                "MessageBytes",
                shared.catalog.limits.message_bytes,
            ));
        }
        let pending = state.take(self.right.id).ok_or(Fault::Closed)?;
        pending.sender.send(result).map_err(|_| Fault::Closed)?;
        shared.changed.notify_waiters();
        Ok(())
    }
}

impl Drop for Right {
    fn drop(&mut self) {
        self.shared.lock().discard(self.id);
        self.shared.changed.notify_waiters();
    }
}

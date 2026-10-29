//! Undelivered connections and terminals retain cleanup ownership until VM conversion succeeds.

use crate::execution::vm::{
    hosted::{HostedState, proc::AcpTerminals},
    worker::Worker,
};
use crate::{SourceLocation, Value, stdlib::AggregateFactory};
use fpas_acp::{RawJson, connection::Fault};
use std::sync::Arc;

pub(super) enum Output {
    Boolean(bool),
    Connection(Connection),
    Host {
        key: &'static str,
        raw: RawJson,
        terminal: Option<Terminal>,
    },
}
pub(super) struct Connection {
    pub id: Option<u64>,
    pub endpoint: Option<u64>,
    pub state: Arc<HostedState>,
}
impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.state.acp.abandon(id);
        }
    }
}
pub(super) struct Terminal {
    pub id: Option<String>,
    pub owner: Arc<AcpTerminals>,
}
impl Drop for Terminal {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.owner.release(&id, None);
        }
    }
}
impl Output {
    pub(super) fn into_value(
        self,
        worker: &Worker,
        location: SourceLocation,
    ) -> Result<Value, Fault> {
        match self {
            Self::Boolean(value) => Ok(Value::Boolean(value)),
            Self::Connection(mut owned) => {
                let id = owned.id.ok_or(Fault::Host)?;
                let value = if let Some(endpoint) = owned.endpoint {
                    worker
                        .layouts
                        .record(
                            "Std.Ai.Acp.MessageConnection",
                            vec![Value::OpaqueHandle(id), Value::OpaqueHandle(endpoint)],
                            location,
                        )
                        .map_err(|_| Fault::Host)?
                } else {
                    Value::OpaqueHandle(id)
                };
                owned.id.take();
                Ok(value)
            }
            Self::Host {
                key,
                raw,
                mut terminal,
            } => {
                let value = worker.acp_host_response(key, &raw, location)?;
                if let Some(owned) = &mut terminal {
                    owned.id.take();
                }
                Ok(value)
            }
        }
    }
}

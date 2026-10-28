//! Terminal draining and cleanup run on the existing connection executor.
use crate::execution::vm::hosted::proc::AcpTerminals;
use fpas_acp::{
    RawJson,
    connection::Fault,
    transport::{Frame, Transport, TransportStatus},
};
use futures::future::BoxFuture;
use std::{sync::Arc, time::Duration};

pub(in super::super) struct OwnedTransport {
    pub inner: Arc<dyn Transport>,
    pub terminals: Arc<AcpTerminals>,
}
impl Transport for OwnedTransport {
    fn receive(&self) -> BoxFuture<'_, Result<Option<Frame>, Fault>> {
        Box::pin(async {
            let next = self.inner.receive();
            tokio::pin!(next);
            loop {
                tokio::select! {
                    result = &mut next => return result,
                    _ = self.terminals.changed.notified() => self.terminals.pump()?,
                    _ = tokio::time::sleep(Duration::from_millis(1)), if self.terminals.active() => self.terminals.pump()?,
                }
            }
        })
    }
    fn send(&self, raw: RawJson) -> BoxFuture<'_, Result<(), Fault>> {
        self.inner.send(raw)
    }
    fn session_closed<'a>(&'a self, session: &'a str) -> BoxFuture<'a, Result<(), Fault>> {
        let terminals = self.terminals.clone();
        let session = session.to_owned();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || terminals.close(Some(&session)))
                .await
                .map_err(|_| Fault::Host)?
        })
    }
    fn finish(&self, grace: u64) -> BoxFuture<'_, Result<TransportStatus, Fault>> {
        Box::pin(async move {
            let terminals = self.terminals.clone();
            let cleanup = tokio::task::spawn_blocking(move || terminals.close(None))
                .await
                .map_err(|_| Fault::Host)
                .and_then(|result| result);
            let result = self.inner.finish(grace).await;
            cleanup?;
            result
        })
    }
}

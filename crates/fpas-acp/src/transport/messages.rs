//! Production message transport: accepted frames retain capacity until consumed.

use crate::{
    RawJson,
    connection::{
        Fault, Limits,
        queue::{Item, Queue},
    },
};
use std::sync::Arc;

/// Application-facing endpoint for a connection carried over an external link.
#[derive(Clone)]
pub struct MessageEndpoint {
    pub(crate) input: Arc<Queue<RawJson>>,
    pub(crate) output: Arc<Queue<RawJson>>,
    maximum: usize,
}

/// Driver-facing half of the owned message transport.
pub struct MessageTransport {
    pub(crate) endpoint: MessageEndpoint,
}

/// Allocate a bounded message pair after validating all resource limits.
pub fn messages(limits: &Limits) -> Result<(MessageEndpoint, MessageTransport), Fault> {
    limits.validate()?;
    let endpoint = MessageEndpoint {
        input: Queue::new(limits.queue_entries, limits.queue_bytes),
        output: Queue::new(limits.queue_entries, limits.queue_bytes),
        maximum: limits.message_bytes,
    };
    Ok((endpoint.clone(), MessageTransport { endpoint }))
}

impl MessageEndpoint {
    /// Admit exactly one complete frame, or return false without taking ownership.
    pub fn feed(&self, frame: RawJson) -> Result<bool, Fault> {
        if frame.text().len() > self.maximum {
            return Err(Fault::LimitExceeded("MessageBytes", self.maximum));
        }
        let bytes = frame.text().len();
        self.input.push(frame, bytes)
    }

    /// Consume one outbound frame; None means pending or drained.
    pub fn try_take(&self) -> Result<Option<RawJson>, Fault> {
        Ok(self.output.pop().map(Item::into_value))
    }

    /// Deliver EOF once after already admitted input.
    pub fn close_input(&self) -> bool {
        self.input.close()
    }

    /// Whether the local driver has closed output after its final admitted frame.
    pub fn output_closed(&self) -> bool {
        self.output.closed()
    }
}

impl MessageTransport {
    async fn send_frame(&self, frame: RawJson) -> Result<(), Fault> {
        let bytes = frame.text().len();
        if bytes > self.endpoint.maximum {
            return Err(Fault::LimitExceeded("MessageBytes", self.endpoint.maximum));
        }
        if bytes > self.endpoint.output.maximum() {
            return Err(Fault::LimitExceeded(
                "OutboundQueue",
                self.endpoint.output.maximum(),
            ));
        }
        loop {
            let changed = self.endpoint.output.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.endpoint.output.push(frame.clone(), bytes)? {
                return Ok(());
            }
            changed.await;
        }
    }
}

impl super::Transport for MessageTransport {
    fn receive(&self) -> futures::future::BoxFuture<'_, Result<Option<super::Frame>, Fault>> {
        Box::pin(async {
            Ok(self.endpoint.input.receive().await.map(|item| {
                let (raw, credit) = item.split();
                super::Frame::credited(raw, credit)
            }))
        })
    }

    fn send(&self, frame: RawJson) -> futures::future::BoxFuture<'_, Result<(), Fault>> {
        Box::pin(async move { self.send_frame(frame).await })
    }

    fn finish(
        &self,
        _grace_millis: u64,
    ) -> futures::future::BoxFuture<'_, Result<super::TransportStatus, Fault>> {
        Box::pin(async {
            self.endpoint.input.close();
            self.endpoint.input.discard();
            self.endpoint.output.close();
            Ok(super::TransportStatus::default())
        })
    }
}

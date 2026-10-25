//! Original-message capture and bounded handoff through the SDK's ordered loop.

use super::{
    Fault,
    shared::{Publication, Shared},
};
use crate::{
    RawJson,
    protocol::v1::RequestId,
    transport::{Frame, Transport},
};
use agent_client_protocol::{Lines, RawJsonRpcMessage};
use futures::{sink, stream};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub(super) struct Receipt {
    pub frame: Frame,
    _acknowledge: oneshot::Sender<()>,
}

pub(super) struct Wire {
    pub shared: Arc<Shared>,
    pub transport: Arc<dyn Transport>,
    captured: Mutex<Option<Receipt>>,
    automatic: Mutex<Option<(RequestId, Receipt)>>,
}

impl Wire {
    pub fn new(shared: Arc<Shared>, transport: Arc<dyn Transport>) -> Arc<Self> {
        Arc::new(Self {
            shared,
            transport,
            captured: Mutex::new(None),
            automatic: Mutex::new(None),
        })
    }

    pub fn take(&self) -> Result<Receipt, Fault> {
        self.captured
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .ok_or(Fault::InvalidMessage)
    }

    pub fn automatic(&self, id: RequestId, receipt: Receipt) {
        *self.automatic.lock().unwrap_or_else(|e| e.into_inner()) = Some((id, receipt));
    }

    pub fn lines(
        self: &Arc<Self>,
    ) -> impl agent_client_protocol::ConnectTo<agent_client_protocol::Client> + 'static {
        let incoming = stream::unfold(
            (self.clone(), None, false),
            async move |(wire, barrier, ended)| {
                if ended {
                    return None;
                }
                if let Some(acknowledged) = barrier {
                    let _ = acknowledged.await;
                }
                match wire.next().await {
                    Ok(Some((line, barrier))) => Some((Ok(line), (wire, barrier, false))),
                    Ok(None) => None,
                    Err(fault) => {
                        wire.shared.fail(fault);
                        Some((
                            Err(std::io::Error::other("ACP input failed")),
                            (wire, None, true),
                        ))
                    }
                }
            },
        );
        let outgoing = sink::unfold(self.clone(), async move |wire, line: String| {
            let result = async {
                let raw = RawJson::parse_bounded(&line, wire.shared.limits.message_bytes)?;
                wire.transport.send(raw.clone()).await?;
                wire.sent(&raw)?;
                Ok::<_, Fault>(())
            }
            .await;
            if let Err(fault) = result {
                wire.shared.fail(fault);
                return Err(std::io::Error::other("ACP output failed"));
            }
            Ok(wire)
        });
        Lines::new(outgoing, incoming)
    }

    async fn next(&self) -> Result<Option<(String, Option<oneshot::Receiver<()>>)>, Fault> {
        loop {
            let Some(frame) = self.transport.receive().await? else {
                self.shared.end_input();
                return Ok(None);
            };
            if frame.raw.text().len() > self.shared.limits.message_bytes {
                return Err(Fault::LimitExceeded(
                    "MessageBytes",
                    self.shared.limits.message_bytes,
                ));
            }
            // SDK envelope validation remains authoritative. ACP input is one message per frame.
            let message: RawJsonRpcMessage =
                serde_json::from_str(frame.raw.text()).map_err(|_| Fault::InvalidMessage)?;
            if let RawJsonRpcMessage::Notification(notification) = &message
                && notification.method.as_ref() == "$/cancel_request"
            {
                let params = frame.raw.member("params")?.ok_or(Fault::InvalidMessage)?;
                crate::codec::deserialize::<crate::protocol::v1::CancelRequestNotification>(
                    &params,
                )?;
            }
            match &message {
                RawJsonRpcMessage::Response(response)
                    if !self
                        .shared
                        .lock()
                        .pending
                        .values()
                        .any(|entry| entry.wire.as_ref() == Some(response_id(response))) =>
                {
                    // The SDK ignores unmatched responses without invoking a handler.
                    // Discard them before its unbounded staging channel, and yield so
                    // a flooding peer cannot monopolize the VM's native executor.
                    drop(message);
                    drop(frame);
                    tokio::task::yield_now().await;
                    continue;
                }
                RawJsonRpcMessage::Request(request) => {
                    if self
                        .shared
                        .lock()
                        .responders
                        .values()
                        .any(|entry| entry.sdk.id() == &request.id)
                    {
                        return Err(Fault::InvalidMessage);
                    }
                    return self.capture(frame);
                }
                _ => return self.capture(frame),
            }
        }
    }

    fn capture(
        &self,
        frame: Frame,
    ) -> Result<Option<(String, Option<oneshot::Receiver<()>>)>, Fault> {
        let line = frame.raw.text().to_owned();
        let (acknowledge, acknowledged) = oneshot::channel();
        *self.captured.lock().unwrap_or_else(|e| e.into_inner()) = Some(Receipt {
            frame,
            _acknowledge: acknowledge,
        });
        Ok(Some((line, Some(acknowledged))))
    }

    fn sent(&self, raw: &RawJson) -> Result<(), Fault> {
        let message: RawJsonRpcMessage =
            serde_json::from_str(raw.text()).map_err(|_| Fault::InvalidMessage)?;
        let cancelled = if matches!(&message, RawJsonRpcMessage::Notification(notification) if notification.method.as_ref() == "$/cancel_request")
        {
            let params = raw.member("params")?.ok_or(Fault::InvalidMessage)?;
            Some(
                crate::codec::deserialize::<crate::protocol::v1::CancelRequestNotification>(
                    &params,
                )?
                .request_id,
            )
        } else {
            None
        };
        let publication = {
            let mut state = self.shared.lock();
            let index = state
                .publications
                .iter()
                .position(|(key, _)| match (key, &message) {
                    (Publication::Cancellation(id), _) => cancelled.as_ref() == Some(id),
                    (Publication::Request(id), RawJsonRpcMessage::Request(request)) => {
                        id == &request.id
                    }
                    (
                        Publication::Notification(method),
                        RawJsonRpcMessage::Notification(notification),
                    ) => method.as_str() == notification.method.as_ref(),
                    (Publication::Response(id), RawJsonRpcMessage::Response(response)) => {
                        id == response_id(response)
                    }
                    _ => false,
                });
            index.and_then(|index| state.publications.remove(index))
        };
        drop(publication);
        if let RawJsonRpcMessage::Response(response) = message {
            let mut automatic = self.automatic.lock().unwrap_or_else(|e| e.into_inner());
            if automatic
                .as_ref()
                .is_some_and(|(id, _)| id == response_id(&response))
            {
                automatic.take();
            }
            drop(automatic);
            // The SDK may reject a protocol-level request before invoking our handler.
            let mut captured = self.captured.lock().unwrap_or_else(|e| e.into_inner());
            if captured.as_ref().is_some_and(|receipt| {
                serde_json::from_str::<RawJsonRpcMessage>(receipt.frame.raw.text()).is_ok_and(|message| matches!(message, RawJsonRpcMessage::Request(request) if &request.id == response_id(&response)))
            }) { captured.take(); }
        }
        Ok(())
    }
}

fn response_id(response: &agent_client_protocol::RawJsonRpcResponse) -> &RequestId {
    match response {
        agent_client_protocol::RawJsonRpcResponse::Result { id, .. }
        | agent_client_protocol::RawJsonRpcResponse::Error { id, .. } => id,
    }
}

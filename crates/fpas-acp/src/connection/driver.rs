//! One SDK task per connection. Callbacks detach data and never await application work.

use super::{
    Closed, Event, Fault, Payload, Responder,
    shared::{Command, Deferred, Publication, RESPONDER_TAG, SERIAL_MASK, Shared},
    wire::Wire,
};
use crate::{
    RawJson, binding,
    transport::{Transport, TransportStatus},
};
use agent_client_protocol::{
    Agent, Client, ConnectionTo, Dispatch, HandleDispatchFrom, Handled, UntypedMessage,
};
use futures::FutureExt;
use std::sync::Arc;

pub(super) async fn run(shared: Arc<Shared>, transport: Arc<dyn Transport>) {
    let wire = Wire::new(shared.clone(), transport.clone());
    let result = std::panic::AssertUnwindSafe(async {
        let sdk = Client
            .builder()
            .with_handler(Handler { wire: wire.clone() })
            .connect_with(wire.lines(), async |cx| commands(shared.clone(), cx).await);
        // SDK graceful draining can wait on physical output. Local close owns a
        // separate cancellation path so cleanup never depends on peer progress.
        tokio::select! {
            result = sdk => result,
            _ = shared.wait_stopping() => Ok(()),
        }
    })
    .catch_unwind()
    .await;
    if !matches!(result, Ok(Ok(()))) {
        shared.fail(Fault::Host);
    }
    let local_close = shared.lock().local_close;
    shared.stop_admission();
    // Drop SDK-owned response state before awaiting process cleanup.
    drop(wire);
    let transport_status = std::panic::AssertUnwindSafe(async {
        transport.finish(shared.limits.shutdown_millis).await
    })
    .catch_unwind()
    .await
    .unwrap_or(Err(Fault::Host));
    let (status, cleanup_fault) = match transport_status {
        Ok(status) => (status, None),
        Err(fault) => (TransportStatus::default(), Some(fault)),
    };
    let process_fault = if !local_close
        && matches!(status.exit, Some(crate::transport::ExitStatus::Exited(code)) if code != 0)
        || !local_close && matches!(status.exit, Some(crate::transport::ExitStatus::Signaled(_)))
    {
        Some(Fault::Host)
    } else {
        None
    };
    let fault = shared
        .lock()
        .fault
        .clone()
        .or(cleanup_fault.clone())
        .or(process_fault);
    shared.finish(Closed {
        cleanup_fault,
        fault,
        transport: status,
    });
}

async fn commands(
    shared: Arc<Shared>,
    cx: ConnectionTo<Agent>,
) -> Result<(), agent_client_protocol::Error> {
    loop {
        let changed = shared.changed.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if shared.stopping() {
            return Ok(());
        }
        if let Some(command) = shared.commands.pop() {
            publish(&shared, &cx, command)?;
            // Let transport/dispatch tasks make progress under sustained local admission.
            tokio::task::yield_now().await;
            continue;
        }
        tokio::select! {
            _ = &mut changed => {},
            _ = cx.incoming_closed() => return Ok(()),
        }
    }
}

fn publish(
    shared: &Arc<Shared>,
    cx: &ConnectionTo<Agent>,
    command: super::queue::Item<Command>,
) -> Result<(), agent_client_protocol::Error> {
    match &command.value {
        Command::Cancel { token } => {
            let cancellation = {
                let state = shared.lock();
                state
                    .pending
                    .get(token)
                    .and_then(|pending| pending.cancellation.clone().zip(pending.wire.clone()))
            };
            if let Some((cancellation, id)) = cancellation {
                shared
                    .lock()
                    .publications
                    .push_back((Publication::Cancellation(id), command));
                cancellation.cancel()?;
            }
        }
        Command::Send { token, method, raw } => {
            let token = *token;
            let prepared = cx.prepare_request(UntypedMessage::new(method, json(raw)?)?);
            let id = prepared.id().clone();
            let cancellation = prepared.cancellation_handle();
            {
                let mut state = shared.lock();
                let Some(pending) = state.pending.get_mut(&token) else {
                    return Ok(());
                };
                pending.wire = Some(id.clone());
                pending.cancellation = Some(cancellation.clone());
                state
                    .publications
                    .push_back((Publication::Request(id), command));
            }
            prepared.on_receiving_result(async |_result| Ok(()))?;
        }
        Command::Notify { method, raw } => {
            let notification = UntypedMessage::new(method, json(raw)?)?;
            let method = method.clone();
            shared
                .lock()
                .publications
                .push_back((Publication::Notification(method), command));
            cx.send_notification(notification)?;
        }
        Command::Reply {
            responder,
            raw,
            error,
        } => {
            let responder = *responder;
            let payload = if *error {
                Err(
                    crate::codec::decode::<agent_client_protocol::Error>(raw.clone())
                        .map_err(|_| sdk_error())?
                        .value,
                )
            } else {
                Ok(json(raw)?)
            };
            let entry = shared.lock().responders.remove(&responder);
            if let Some(entry) = entry {
                let id = entry.sdk.id().clone();
                shared
                    .lock()
                    .publications
                    .push_back((Publication::Response(id), command));
                entry.sdk.respond_with_result(payload)?;
            }
        }
    }
    Ok(())
}

struct Handler {
    wire: Arc<Wire>,
}

impl HandleDispatchFrom<Agent> for Handler {
    async fn handle_dispatch_from(
        &mut self,
        message: Dispatch,
        _cx: ConnectionTo<Agent>,
    ) -> Result<Handled<Dispatch>, agent_client_protocol::Error> {
        let result = self.dispatch(message);
        if let Err(fault) = result {
            self.wire.shared.fail(fault);
            return Err(sdk_error());
        }
        Ok(Handled::Yes)
    }
    fn describe_chain(&self) -> impl std::fmt::Debug {
        "FPAS detached ACP dispatch"
    }
}

impl Handler {
    fn dispatch(&self, message: Dispatch) -> Result<(), Fault> {
        let receipt = self.wire.take()?;
        let raw = receipt.frame.raw.clone();
        let shared = &self.wire.shared;
        let version = shared.version;
        let catalog = binding::catalog(version)?;
        match message {
            Dispatch::Request(request, responder) => {
                // Install the SDK cancellation marker before exposing the response right.
                let cancellation = responder.cancellation();
                let params = raw.member("params")?.unwrap_or(RawJson::parse("null")?);
                let payload = match catalog.decode_message("AgentRequest", request.method(), params)
                {
                    Ok(decoded) => decoded.payload,
                    Err(_) => {
                        self.wire.automatic(responder.id().clone(), receipt);
                        let error = if catalog.supports_method("AgentRequest", request.method()) {
                            agent_client_protocol::Error::invalid_params()
                        } else {
                            agent_client_protocol::Error::method_not_found()
                        };
                        responder
                            .respond_with_error(error)
                            .map_err(|_| Fault::Host)?;
                        return Ok(());
                    }
                };
                let token = {
                    let id_bytes = serde_json::to_string(responder.id())
                        .map_err(|_| Fault::InvalidMessage)?
                        .len();
                    let metadata = shared.incoming_metadata.reserve(
                        request
                            .method()
                            .len()
                            .saturating_add(id_bytes)
                            .saturating_mul(2)
                            .saturating_add(512),
                    )?;
                    let mut state = shared.lock();
                    Shared::active(&state)?;
                    if state.responders.len() == shared.limits.incoming_requests {
                        return Err(Fault::LimitExceeded(
                            "IncomingRequests",
                            shared.limits.incoming_requests,
                        ));
                    }
                    if state.responder_serial == SERIAL_MASK {
                        return Err(Fault::LimitExceeded("Responders", SERIAL_MASK as usize));
                    }
                    state.responder_serial += 1;
                    let token = Responder(shared.identity(RESPONDER_TAG, state.responder_serial));
                    state.responders.insert(
                        token,
                        Deferred {
                            _metadata: metadata,
                            sdk: responder,
                            cancellation,
                            queued: false,
                        },
                    );
                    token
                };
                shared.event(Event {
                    payload: Payload::Request(token, version, payload),
                    raw: Some(raw),
                })?;
            }
            Dispatch::Notification(notification) => {
                let params = raw.member("params")?.unwrap_or(RawJson::parse("null")?);
                if notification.method() == "$/cancel_request" {
                    let cancelled: crate::protocol::v1::CancelRequestNotification =
                        crate::codec::deserialize(&params)?;
                    let token = shared
                        .lock()
                        .responders
                        .iter()
                        .find(|(_, entry)| {
                            entry.sdk.id() == &cancelled.request_id
                                && entry.cancellation.is_cancelled()
                        })
                        .map(|(token, _)| *token);
                    if let Some(token) = token {
                        shared.cancel_incoming(token, raw)?;
                    }
                    return Ok(());
                }
                let payload = match catalog.decode_message(
                    "AgentNotification",
                    notification.method(),
                    params.clone(),
                ) {
                    Ok(decoded) => Payload::Notification(version, decoded.payload),
                    Err(_)
                        if !catalog.supports_method("AgentNotification", notification.method()) =>
                    {
                        Payload::UnknownNotification(notification.method().into(), params)
                    }
                    Err(_) => return Err(Fault::InvalidMessage),
                };
                shared.event(Event {
                    payload,
                    raw: Some(raw),
                })?;
            }
            Dispatch::Response(result, router) => {
                let pending = {
                    let state = shared.lock();
                    state
                        .pending
                        .iter()
                        .find(|(_, pending)| pending.wire.as_ref() == Some(router.id()))
                        .map(|(token, pending)| (*token, pending.method.clone()))
                };
                if let Some((token, method)) = pending {
                    let payload = if let Some(error) = raw.member("error")? {
                        Payload::RequestFailed(token, Fault::Peer(error))
                    } else {
                        let response = raw.member("result")?.ok_or(Fault::InvalidMessage)?;
                        let decoded = catalog.decode_message("AgentResponse", &method, response)?;
                        Payload::Response(token, version, decoded.payload)
                    };
                    shared.response(
                        token,
                        Event {
                            payload,
                            raw: Some(raw),
                        },
                    )?;
                }
                router.route_with_result(result).map_err(|_| Fault::Host)?;
            }
        }
        Ok(())
    }
}

fn json(raw: &RawJson) -> Result<serde_json::Value, agent_client_protocol::Error> {
    serde_json::from_str(raw.text()).map_err(|_| sdk_error())
}
fn sdk_error() -> agent_client_protocol::Error {
    agent_client_protocol::Error::internal_error()
}

//! One SDK task per connection. Callbacks detach data and never await application work.
mod handler;
use super::{client, host, mcp};
use handler::Handler;

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
        Command::RetireRoute { tokens, responders } => {
            let tokens = tokens.clone();
            let responders = responders.clone();
            let command = Arc::new(command);
            for token in tokens {
                let cancellation = {
                    let mut state = shared.lock();
                    if let Some(pending) = state.pending.get_mut(&token)
                        && !pending.cancel_requested
                    {
                        pending.cancel_requested = true;
                        pending.cancellation.clone().zip(pending.wire.clone())
                    } else {
                        None
                    }
                };
                if let Some((cancellation, id)) = cancellation {
                    shared
                        .lock()
                        .publications
                        .push_back((Publication::Cancellation(id), Box::new(command.clone())));
                    cancellation.cancel()?;
                }
            }
            super::client::cancellation::settle(shared, &responders).map_err(|_| sdk_error())?;
        }
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
                    .push_back((Publication::Cancellation(id), Box::new(command)));
                cancellation.cancel()?;
            }
        }
        Command::Send { token, method, raw } => {
            let token = *token;
            let retired = {
                let state = shared.lock();
                state.pending.get(&token).is_some_and(|pending| matches!(pending.action, super::client::Action::Mcp(id) if !state.routes.values().any(|endpoint| endpoint.id == id)))
            };
            if retired {
                shared
                    .response(
                        token,
                        Event {
                            payload: Payload::RequestFailed(token, Fault::Cancelled),
                            raw: None,
                        },
                    )
                    .map_err(|_| sdk_error())?;
                return Ok(());
            }
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
                    .push_back((Publication::Request(id), Box::new(command)));
            }
            prepared.on_receiving_result(async |_result| Ok(()))?;
        }
        Command::Notify {
            method,
            raw,
            cancelled,
        } => {
            let notification = UntypedMessage::new(method, json(raw)?)?;
            let method = method.clone();
            let cancelled = cancelled.clone();
            shared
                .lock()
                .publications
                .push_back((Publication::Notification(method), Box::new(command)));
            cx.send_notification(notification)?;
            super::client::cancellation::settle(shared, &cancelled).map_err(|_| sdk_error())?;
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
                    .push_back((Publication::Response(id), Box::new(command)));
                entry.sdk.respond_with_result(payload)?;
            }
        }
    }
    Ok(())
}

fn json(raw: &RawJson) -> Result<serde_json::Value, agent_client_protocol::Error> {
    serde_json::from_str(raw.text()).map_err(|_| sdk_error())
}
fn sdk_error() -> agent_client_protocol::Error {
    agent_client_protocol::Error::internal_error()
}

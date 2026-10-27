//! SDK dispatch converts messages into detached, bounded events.
use super::*;
use crate::binding::CatalogCodec as _;
pub(super) struct Handler {
    pub(super) wire: Arc<Wire>,
}

impl HandleDispatchFrom<Agent> for Handler {
    async fn handle_dispatch_from(
        &mut self,
        message: Dispatch,
        _cx: ConnectionTo<Agent>,
    ) -> Result<Handled<Dispatch>, agent_client_protocol::Error> {
        let result = self.dispatch(message).await;
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
    async fn dispatch(&self, message: Dispatch) -> Result<(), Fault> {
        let receipt = self.wire.take()?;
        let raw = receipt.frame.raw.clone();
        let shared = &self.wire.shared;
        let version = shared.family();
        let catalog = binding::catalog(version)?;
        match message {
            Dispatch::Request(request, responder) => {
                // Install the SDK cancellation marker before exposing the response right.
                let cancellation = responder.cancellation();
                let params = raw.member("params")?.unwrap_or(RawJson::parse("null")?);
                let payload = match catalog.decode_message(
                    "AgentRequest",
                    request.method(),
                    params.clone(),
                ) {
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
                let session =
                    if !request.method().starts_with('_') && request.method() != "mcp/message" {
                        params
                            .member("sessionId")?
                            .map(|id| crate::codec::deserialize::<String>(&id))
                            .transpose()?
                    } else {
                        None
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
                            .saturating_add(params.text().len().saturating_mul(16))
                            .saturating_add(
                                session.as_ref().map_or(0, |id| id.len().saturating_mul(4)),
                            )
                            .saturating_add(512),
                    )?;
                    let mut state = shared.lock();
                    Shared::active(&state)?;
                    {
                        let lifecycle = state.lifecycle;
                        let request_scope = params.member("requestId")?.is_some_and(|id| {
                            crate::codec::deserialize::<crate::protocol::v1::RequestId>(&id)
                                .is_ok_and(|id| {
                                    state
                                        .pending
                                        .values()
                                        .any(|pending| pending.wire.as_ref() == Some(&id))
                                })
                        });
                        let allowed = if version == crate::schema::Version::V1 {
                            state.client.allow_incoming(
                                lifecycle,
                                request.method(),
                                &params,
                                request_scope,
                            )?
                        } else {
                            state.client.allow_incoming_v2(
                                lifecycle,
                                request.method(),
                                &params,
                                request_scope,
                            )?
                        };
                        let cancelled = request.method() == "session/request_permission"
                            && session
                                .as_ref()
                                .is_some_and(|id| state.client.cancelled(id));
                        let allowed = allowed
                            && (request.method() != "mcp/message"
                                || super::mcp::route(&state, &params, false).is_ok());
                        if !allowed || cancelled {
                            drop(state);
                            self.wire.automatic(responder.id().clone(), receipt);
                            if cancelled {
                                responder
                                    .respond_with_result(Ok(
                                        serde_json::json!({"outcome":{"outcome":"cancelled"}}),
                                    ))
                                    .map_err(|_| Fault::Host)?;
                            } else {
                                responder
                                    .respond_with_error(
                                        agent_client_protocol::Error::invalid_request(),
                                    )
                                    .map_err(|_| Fault::Host)?;
                            }
                            return Ok(());
                        }
                    }
                    if state.responders.len() + state.cancelled_replies.len()
                        == shared.limits.incoming_requests
                    {
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
                    let route = if request.method() == "mcp/message" {
                        Some(super::mcp::route(&state, &params, false)?)
                    } else {
                        None
                    };
                    state.responders.insert(
                        token,
                        Deferred {
                            version,
                            route,
                            params,
                            execution: super::host::Execution::Available,
                            session,
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
                let supported = shared.lock().client.experimental_incoming(
                    notification.method(),
                    &params,
                    version,
                )?;
                if supported {
                    let mut state = shared.lock();
                    let lifecycle = state.lifecycle;
                    state.client.notification(
                        lifecycle,
                        notification.method(),
                        &params,
                        &shared.limits,
                    )?;
                }
                let payload = if !supported {
                    Payload::UnknownNotification(notification.method().into(), params.clone())
                } else {
                    match catalog.decode_message(
                        "AgentNotification",
                        notification.method(),
                        params.clone(),
                    ) {
                        Ok(mut decoded) => {
                            shared
                                .lock()
                                .client
                                .filter_usage(&mut decoded.payload, version);
                            Payload::Notification(version, decoded.payload)
                        }
                        Err(_)
                            if !catalog
                                .supports_method("AgentNotification", notification.method()) =>
                        {
                            Payload::UnknownNotification(notification.method().into(), params)
                        }
                        Err(_) => return Err(Fault::InvalidMessage),
                    }
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
                        .map(|(token, pending)| {
                            (
                                *token,
                                pending.method.clone(),
                                pending.version,
                                match &pending.action {
                                    super::client::Action::Close(id) => Some(id.clone()),
                                    _ => None,
                                },
                            )
                        })
                };
                if let Some((token, method, version, session)) = pending {
                    let catalog = binding::catalog(version)?;
                    let payload = if let Some(error) = raw.member("error")? {
                        Payload::RequestFailed(token, Fault::Peer(error))
                    } else if method == "initialize" && shared.version == crate::schema::Version::V2
                    {
                        let response = raw.member("result")?.ok_or(Fault::InvalidMessage)?;
                        let selected: u16 = crate::codec::deserialize(
                            &response
                                .member("protocolVersion")?
                                .ok_or(Fault::InvalidMessage)?,
                        )?;
                        if selected == 1 || selected == 2 {
                            let selected = if selected == 1 {
                                crate::schema::Version::V1
                            } else {
                                crate::schema::Version::V2
                            };
                            Payload::Initialized(
                                token,
                                selected,
                                binding::catalog(selected)?
                                    .decode("InitializeResponse", &response)?,
                            )
                        } else {
                            Payload::RequestFailed(token, Fault::UnsupportedVersion(selected))
                        }
                    } else {
                        let response = raw.member("result")?.ok_or(Fault::InvalidMessage)?;
                        if method == "mcp/message" {
                            Payload::McpResponse(
                                token,
                                version,
                                catalog.decode("MessageMcpResponse", &response)?,
                            )
                        } else {
                            let mut decoded =
                                catalog.decode_message("AgentResponse", &method, response)?;
                            shared
                                .lock()
                                .client
                                .filter_usage(&mut decoded.payload, version);
                            Payload::Response(token, version, decoded.payload)
                        }
                    };
                    if let Some(session) = session
                        && matches!(payload, Payload::Response(_, _, _))
                    {
                        let sessions = shared.lock().client.closing_tree(&session, token);
                        for session in sessions {
                            self.wire.session_closed(&session).await?;
                        }
                    }
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

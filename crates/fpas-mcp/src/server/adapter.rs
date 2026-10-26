//! The official SDK performs wire handling; application calls leave its executor.

use super::{
    Endpoint,
    state::{CallGuard, Data, Pending},
};
use crate::{Fault, model::*};
use rmcp::{
    ServerHandler,
    service::{RequestContext, RoleServer},
};
use serde_json::Value;
use tokio::sync::oneshot;

impl Endpoint {
    fn admit(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<(u64, oneshot::Receiver<Result<CallToolResult, ErrorData>>), Fault> {
        if request.input_responses.is_some() || request.request_state.is_some() {
            return Err(Fault::InvalidArgument("Continuation"));
        }
        let limits = &self.shared.catalog.limits;
        let metadata_size =
            serde_json::to_value(&context.meta).map_err(|_| Fault::InvalidArguments)?;
        // The SDK moves request metadata into its context before invoking handlers.
        let mut message = serde_json::to_value(&request).map_err(|_| Fault::InvalidArguments)?;
        if !metadata_size
            .as_object()
            .is_some_and(|object| object.is_empty())
        {
            message
                .as_object_mut()
                .ok_or(Fault::InvalidArguments)?
                .insert("_meta".into(), metadata_size.clone());
        }
        let serialized = serde_json::to_vec(&message)
            .map_err(|_| Fault::InvalidArguments)?
            .len();
        if serialized > limits.message_bytes {
            return Err(Fault::LimitExceeded("MessageBytes", limits.message_bytes));
        }
        let arguments = request.arguments.unwrap_or_default();
        let mut state = self.shared.lock();
        state.active()?;
        if !state.endpoints.contains_key(&self.id) || context.ct.is_cancelled() {
            return Err(Fault::Closed);
        }
        let (connection_id, session_id) = match &state.endpoints[&self.id].binding {
            super::state::Binding::Standalone => (None, None),
            super::state::Binding::Pending(_) => {
                return Err(Fault::InvalidArgument("AttachmentPending"));
            }
            super::state::Binding::Session(connection, session) => {
                (Some(*connection), Some(session.clone()))
            }
        };
        if state.pending.len() == limits.pending_calls {
            return Err(Fault::LimitExceeded("PendingCalls", limits.pending_calls));
        }
        let bytes = retained_size(&Value::Object(arguments.clone()))
            .saturating_add(retained_size(&metadata_size))
            .saturating_add(request.name.len())
            .saturating_add(context.id.to_string().len())
            .saturating_add(session_id.as_ref().map_or(0, String::len))
            .saturating_add(1024);
        if bytes.saturating_add(state.bytes) > limits.pending_bytes {
            return Err(Fault::LimitExceeded("PendingBytes", limits.pending_bytes));
        }
        let handler = self
            .shared
            .catalog
            .validate_arguments(&request.name, &arguments)?;
        let id = state.next_call.checked_add(1).ok_or(Fault::Closed)?;
        let protocol_version = context.protocol_version();
        let data = Data {
            connection_id,
            session_id,
            handler,
            name: request.name.to_string(),
            arguments,
            request_id: context.id,
            metadata: context.meta,
            protocol_version,
        };
        let (sender, receiver) = oneshot::channel();
        state.next_call = id;
        state.bytes += bytes;
        state.pending.insert(
            id,
            Pending {
                endpoint: self.id,
                name: data.name.clone(),
                data: Some(data),
                sender,
                cancellation: context.ct,
                bytes,
            },
        );
        state.queue.push_back(id);
        self.shared.changed.notify_waiters();
        Ok((id, receiver))
    }
}

impl ServerHandler for Endpoint {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("fpas-mcp", env!("CARGO_PKG_VERSION")))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        let state = self.shared.lock();
        if state.closed || !state.endpoints.contains_key(&self.id) {
            return None;
        }
        self.shared.catalog.tool(name).cloned()
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let state = self.shared.lock();
        state.active().map_err(protocol_error)?;
        if !state.endpoints.contains_key(&self.id) {
            return Err(protocol_error(Fault::Closed));
        }
        if request.is_some_and(|request| request.cursor.is_some()) {
            return Err(ErrorData::invalid_params(
                "MCP catalog has no continuation cursor",
                None,
            ));
        }
        Ok(ListToolsResult::with_all_items(
            self.shared.catalog.tools().cloned().collect(),
        ))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let cancellation = context.ct.clone();
        let (id, receiver) = match self.admit(request, context) {
            Ok(admitted) => admitted,
            Err(Fault::InvalidArguments) => {
                return Ok(CallToolResult::error(vec![ContentBlock::text(
                    Fault::InvalidArguments.to_string(),
                )])
                .into());
            }
            Err(error) => return Err(protocol_error(error)),
        };
        let _guard = CallGuard {
            shared: &self.shared,
            id,
        };
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(protocol_error(Fault::Closed)),
            _ = self.cancellation.cancelled() => Err(protocol_error(Fault::Closed)),
            result = receiver => result.map_err(|_| protocol_error(Fault::Closed))?.map(Into::into),
        }
    }
}

fn protocol_error(error: Fault) -> ErrorData {
    match error {
        Fault::UnknownTool | Fault::InvalidArgument(_) => {
            ErrorData::invalid_params(error.to_string(), None)
        }
        _ => ErrorData::internal_error(error.to_string(), None),
    }
}

fn retained_size(value: &Value) -> usize {
    std::mem::size_of::<Value>().saturating_add(match value {
        Value::String(text) => text.len(),
        Value::Number(number) => number.to_string().len(),
        Value::Array(values) => values
            .iter()
            .map(retained_size)
            .fold(0usize, usize::saturating_add),
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| {
                key.len()
                    .saturating_add(64)
                    .saturating_add(retained_size(value))
            })
            .fold(0usize, usize::saturating_add),
        _ => 0,
    })
}

//! Native-only suspension ownership; FPAS callbacks enter the serving task's frames.

use super::{Failure, arguments as args, registry::Claim};
use crate::execution::vm::{
    VmError,
    tasks::{TaskClock, TaskSuspension, TaskSuspensionState},
    worker::Worker,
};
use crate::{FunctionId, InstructionAddress, Intrinsic, McpIntrinsic as Op, Register, Value};
use fpas_mcp::{CleanupReceipt, Fault, PublishTicket, Reader};
use std::{sync::Arc, time::Duration};

pub(in crate::execution::vm) enum Wait {
    Publish {
        ticket: PublishTicket,
        server: u64,
        destination: Option<usize>,
    },
    Close {
        receipts: Vec<CleanupReceipt>,
        id: Option<u64>,
        endpoint: bool,
        release: bool,
        first: bool,
        destination: Option<usize>,
    },
    Serve(ServeWait),
}
pub(in crate::execution::vm) struct ServeWait {
    reader: Reader,
    server: u64,
    token: Option<u64>,
    deadline: Option<u64>,
    looped: bool,
    destination: Option<usize>,
    pub(in crate::execution::vm) origin: (FunctionId, InstructionAddress),
}
pub(in crate::execution::vm) struct Invocation {
    wait: ServeWait,
    claim: Arc<Claim>,
}
impl Invocation {
    pub(in crate::execution::vm) fn origin(&self) -> (FunctionId, InstructionAddress) {
        self.wait.origin
    }
}
impl Wait {
    pub(in crate::execution::vm) fn debug_state(&self, clock: &TaskClock) -> TaskSuspensionState {
        if let Self::Serve(ServeWait {
            deadline: Some(deadline),
            ..
        }) = self
        {
            TaskSuspensionState::Sleeping {
                remaining: Duration::from_millis(deadline.saturating_sub(clock.now_millis())),
            }
        } else {
            TaskSuspensionState::Waiting
        }
    }
}
impl Worker {
    pub(in crate::execution::vm) fn acp_detach_tools(
        &mut self,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let result = (|| -> Result<_, Failure> {
            let [owner, server] = arguments else {
                return Err(Fault::InvalidArgument("Arguments").into());
            };
            let connection = self
                .hosted
                .acp
                .connection(args::handle(owner)?)
                .map_err(|_| Fault::Closed)?;
            connection
                .detach_tool_server(&format!("fpas_tools_{}", args::handle(server)?))
                .map_err(|_| Fault::Closed.into())
        })();
        match result {
            Ok((first, receipts)) => {
                let destination = destination.map(|r| self.base + usize::from(r.get()));
                self.suspend_mcp(Wait::Close {
                    receipts,
                    id: None,
                    endpoint: false,
                    release: false,
                    first,
                    destination,
                })?;
                Ok(Some(None))
            }
            Err(e) => Ok(Some(Some(
                self.mcp_result(Err(e), self.intrinsic_location())?,
            ))),
        }
    }
    pub(in crate::execution::vm) fn mcp_wait_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Mcp(op) = intrinsic else {
            return Ok(None);
        };
        if !op.waits() {
            return Ok(None);
        }
        let destination = destination.map(|r| self.base + usize::from(r.get()));
        if let (Op::Serve | Op::ServeOne, [server, ..]) = (op, arguments)
            && let Ok(handlers) = args::handle(server).and_then(|id| self.hosted.mcp.handlers(id))
        {
            for handler in handlers {
                self.validate_callback(&handler, 2)?;
            }
        }
        let result = (|| -> Result<Wait, Failure> {
            match (op, arguments) {
                (Op::Publish, [server, options]) => {
                    let server = args::handle(server)?;
                    Ok(Wait::Publish {
                        ticket: self.hosted.mcp.publish(server, args::http(options)?)?,
                        server,
                        destination,
                    })
                }
                (Op::CloseEndpoint | Op::Close | Op::Release, [owner]) => {
                    let id = args::handle(owner)?;
                    let endpoint = op == Op::CloseEndpoint;
                    let (first, receipts) = if endpoint {
                        self.hosted.mcp.close_endpoint(id)?
                    } else {
                        self.hosted.mcp.close(id)?
                    };
                    Ok(Wait::Close {
                        receipts,
                        id: Some(id),
                        endpoint,
                        release: op == Op::Release,
                        first,
                        destination,
                    })
                }
                (Op::ServeOne, [server, timeout, cancellation]) => {
                    self.mcp_serve_wait(server, Some(timeout), cancellation, destination)
                }
                (Op::Serve, [server, cancellation]) => {
                    self.mcp_serve_wait(server, None, cancellation, destination)
                }
                _ => Err(Fault::InvalidArgument("Arguments").into()),
            }
        })();
        match result {
            Ok(wait) => {
                self.suspend_mcp(wait)?;
                Ok(Some(None))
            }
            Err(error) => Ok(Some(Some(
                self.mcp_result(Err(error), self.intrinsic_location())?,
            ))),
        }
    }
    fn mcp_serve_wait(
        &self,
        server: &Value,
        timeout: Option<&Value>,
        cancellation: &Value,
        destination: Option<usize>,
    ) -> Result<Wait, Failure> {
        let server = args::handle(server)?;
        let looped = timeout.is_none();
        let timeout = timeout
            .map(args::optional)
            .transpose()?
            .flatten()
            .map(args::integer)
            .transpose()?;
        let token = args::optional(cancellation)?
            .map(args::handle)
            .transpose()?;
        if let Some(token) = token {
            self.hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|_| Fault::InvalidArgument("CancellationToken"))?;
        }
        let reader = self.hosted.mcp.server(server)?.reader()?;
        Ok(Wait::Serve(ServeWait {
            reader,
            server,
            token,
            deadline: timeout.map(|v| {
                self.task_clock_ref()
                    .deadline_after(Duration::from_millis(v as u64))
            }),
            looped,
            destination,
            origin: (self.function, self.current_address),
        }))
    }
    fn suspend_mcp(&mut self, wait: Wait) -> Result<(), VmError> {
        self.task_suspension = Some(TaskSuspension::Mcp(Box::new(wait)));
        self.suspend_requested = true;
        if self.poll_task_suspension()? {
            return Ok(());
        }
        if !self.debug_tasks && self.task_id == 0 {
            loop {
                self.wait_observing_scopes()?;
                if self.poll_task_suspension()? {
                    break;
                }
            }
        }
        Ok(())
    }
    pub(in crate::execution::vm) fn poll_mcp_wait(
        &mut self,
        mut wait: Wait,
    ) -> Result<bool, VmError> {
        let location = self.intrinsic_location();
        let completed = match &mut wait {
            Wait::Publish {
                ticket,
                server,
                destination,
            } => ticket.try_complete().map(|result| {
                (
                    *destination,
                    result.map_err(Failure::from).and_then(|p| {
                        self.hosted
                            .mcp
                            .published(*server, p)
                            .map(Value::OpaqueHandle)
                            .map_err(Into::into)
                    }),
                )
            }),
            Wait::Close {
                receipts,
                id,
                endpoint,
                release,
                first,
                destination,
            } => {
                let results: Option<Vec<_>> = receipts.iter().map(CleanupReceipt::result).collect();
                results.map(|results| {
                    let result = results
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                        .map(|_| Value::Boolean(*release || *first))
                        .map_err(Failure::from);
                    if let Some(id) = id {
                        self.hosted.mcp.finish_close(*id, *endpoint, *release);
                    }
                    (*destination, result)
                })
            }
            Wait::Serve(serve) => {
                let cancelled = serve
                    .token
                    .map(|t| {
                        self.hosted
                            .cancellations
                            .is_cancelled(t)
                            .map_err(|_| Fault::InvalidArgument("CancellationToken"))
                    })
                    .transpose();
                match cancelled {
                    Err(e) => Some((serve.destination, Err(e.into()))),
                    Ok(Some(true)) => Some((serve.destination, Err(Failure::Cancelled))),
                    _ => match serve.reader.try_receive() {
                        Ok(Some(call)) => {
                            let Wait::Serve(serve) = wait else {
                                unreachable!()
                            };
                            self.start_mcp_call(serve, call)?;
                            return Ok(true);
                        }
                        Err(Fault::Closed) => Some((serve.destination, Ok(Value::Boolean(true)))),
                        Err(e) => Some((serve.destination, Err(e.into()))),
                        Ok(None)
                            if serve
                                .deadline
                                .is_some_and(|d| self.task_clock_ref().now_millis() >= d) =>
                        {
                            Some((serve.destination, Ok(Value::Boolean(false))))
                        }
                        Ok(None) => None,
                    },
                }
            }
        };
        if let Some((destination, result)) = completed {
            let value = self.mcp_result(result, location)?;
            if let Some(destination) = destination {
                self.store_register(destination, value)?;
            }
            Ok(true)
        } else {
            self.task_suspension = Some(TaskSuspension::Mcp(Box::new(wait)));
            Ok(false)
        }
    }
    fn start_mcp_call(&mut self, wait: ServeWait, call: fpas_mcp::Call) -> Result<(), VmError> {
        let location = self.intrinsic_location();
        let handler = self.hosted.mcp.handler(wait.server, call.handler);
        let handler = match handler {
            Ok(h) => h,
            Err(e) => return self.finish_mcp_serve(wait, Err(e.into())),
        };
        let callback = self.resolve_callback(&handler, 2)?;
        let claim = match self.hosted.mcp.claim(wait.server, call.responder.clone()) {
            Ok(c) => c,
            Err(e) => return self.finish_mcp_serve(wait, Err(e.into())),
        };
        let context = self.mcp_context(wait.server, claim.id, &call, location)?;
        let arguments = self.mcp_raw(
            &serde_json::Value::Object(call.arguments).to_string(),
            location,
        )?;
        self.start_mcp_callback(
            callback,
            context,
            arguments,
            Box::new(Invocation { wait, claim }),
        )
    }
    fn finish_mcp_serve(
        &mut self,
        wait: ServeWait,
        result: Result<Value, Failure>,
    ) -> Result<(), VmError> {
        let value = self.mcp_result(result, self.intrinsic_location())?;
        if let Some(destination) = wait.destination {
            self.store_register(destination, value)?;
        }
        Ok(())
    }
    pub(in crate::execution::vm) fn finish_mcp_callback(
        &mut self,
        invocation: Invocation,
        value: Value,
    ) -> Result<(), VmError> {
        let Invocation { wait, claim } = invocation;
        let result = match value {
            Value::ResultOk(value) => (|| {
                let f = args::record(&value, "Std.Mcp.Types.ToolResult", 1)?;
                let result =
                    serde_json::from_value(args::json(&f[0])?).map_err(|_| Fault::InvalidResult)?;
                claim.responder.respond(Ok(result))
            })(),
            Value::ResultError(_) => {
                claim
                    .responder
                    .respond(Err(fpas_mcp::model::ErrorData::internal_error(
                        "Tool handler failed",
                        None,
                    )))
            }
            _ => Err(Fault::InvalidResult),
        };
        let result = match result {
            Err(Fault::Closed) => Ok(()),
            Err(error) => {
                let _ = claim
                    .responder
                    .respond(Err(fpas_mcp::model::ErrorData::internal_error(
                        "Invalid tool result",
                        None,
                    )));
                Err(error)
            }
            result => result,
        };
        drop(claim);
        if let Err(error) = result {
            return self.finish_mcp_serve(wait, Err(error.into()));
        }
        if wait.looped {
            self.suspend_mcp(Wait::Serve(wait))
        } else {
            self.finish_mcp_serve(wait, Ok(Value::Boolean(true)))
        }
    }
}

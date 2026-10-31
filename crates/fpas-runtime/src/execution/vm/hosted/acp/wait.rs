//! Cooperative receive/close ownership shared by pool tasks and ordinary debugger tasks.

use super::arguments as args;
use crate::execution::vm::{
    VmError,
    tasks::{TaskClock, TaskSuspension, TaskSuspensionState},
    worker::Worker,
};
use crate::{AcpIntrinsic as Op, Intrinsic, Register, SourceLocation, Value};
use fpas_acp::connection::{Connection, Fault, Reader};
use std::time::Duration;

pub(in crate::execution::vm) enum Wait {
    Native(super::native::wait::Wait),
    Receive {
        reader: Reader,
        token: Option<u64>,
        deadline: Option<u64>,
        destination: Option<Register>,
    },
    Close {
        connection: Connection,
        first: bool,
        release: bool,
        destination: Option<Register>,
    },
}

/// A close already initiated by a cancelled task must finish before its owner retires.
pub(in crate::execution::vm) struct Retirement {
    connection: Connection,
    release: bool,
}

impl Wait {
    pub(in crate::execution::vm) fn into_retirement(self) -> Option<Retirement> {
        match self {
            Self::Close {
                connection,
                release,
                ..
            } => Some(Retirement {
                connection,
                release,
            }),
            _ => None,
        }
    }
}

impl Retirement {
    pub(in crate::execution::vm) fn poll(&self, worker: &Worker) -> Result<bool, VmError> {
        if !self.connection.is_closed() {
            return Ok(false);
        }
        worker.hosted.mcp.release_owned(self.connection.id());
        let outcome = self.connection.cleanup_result().and_then(|()| {
            if self.release {
                match worker.hosted.acp.release(&self.connection) {
                    Ok(_) | Err(Fault::Closed) => Ok(()),
                    Err(error) => Err(error),
                }
            } else {
                Ok(())
            }
        });
        outcome.map(|()| true).map_err(|error| {
            worker.runtime_error(
                fpas_diagnostics::codes::RUNTIME_RESOURCE_RELEASE_FAILURE,
                format!("ACP resource retirement failed: {error}"),
                "Inspect the connection cleanup failure.",
            )
        })
    }
}

impl Wait {
    pub(in crate::execution::vm) fn debug_state(&self, clock: &TaskClock) -> TaskSuspensionState {
        match self {
            Self::Receive {
                deadline: Some(deadline),
                ..
            } => TaskSuspensionState::Sleeping {
                remaining: Duration::from_millis(deadline.saturating_sub(clock.now_millis())),
            },
            _ => TaskSuspensionState::Waiting,
        }
    }
}

impl Worker {
    pub(in crate::execution::vm) fn acp_wait_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Acp(operation) = intrinsic else {
            return Ok(None);
        };
        if operation == Op::ServeTools {
            return self.acp_serve_tools(arguments, destination);
        }
        if operation == Op::DetachMcpServer {
            return self.acp_detach_tools(arguments, destination);
        }
        if operation == Op::HostWaitForTerminalExit {
            self.start_acp_native(operation, arguments, destination)?;
            return Ok(Some(None));
        }
        if !operation.waits() {
            return Ok(None);
        }
        let location = self.intrinsic_location();
        let wait = (|| match (operation, arguments) {
            (Op::Receive, [owner, timeout, cancellation]) => {
                let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                let timeout = args::optional(timeout)?
                    .map(|value| match value {
                        Value::Integer(value) if *value >= 0 => Ok(*value as u64),
                        _ => Err(Fault::InvalidArgument("TimeoutMillis")),
                    })
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
                let reader = connection.reader()?;
                let deadline = timeout.map(|value| {
                    self.task_clock_ref()
                        .deadline_after(Duration::from_millis(value))
                });
                Ok(Wait::Receive {
                    reader,
                    token,
                    deadline,
                    destination,
                })
            }
            (Op::Close | Op::Release, [owner]) => {
                let connection = self.hosted.acp.connection(args::handle(owner)?)?;
                let first = connection.close()?;
                Ok(Wait::Close {
                    connection,
                    first,
                    release: operation == Op::Release,
                    destination,
                })
            }
            _ => Err(Fault::InvalidArgument("Arguments")),
        })();
        let wait = match wait {
            Ok(wait) => wait,
            Err(error) => {
                return Ok(Some(Some(self.acp_result(
                    &operation.source_name(),
                    Err(error),
                    location,
                )?)));
            }
        };
        self.suspend_acp_wait(wait)?;
        Ok(Some(None))
    }

    pub(in super::super) fn suspend_acp_wait(&mut self, wait: Wait) -> Result<(), VmError> {
        // The main lane uses the same saved suspension path as tasks and the debugger.
        self.task_suspension = Some(TaskSuspension::Acp(Box::new(wait)));
        self.suspend_requested = true;
        if self.poll_task_suspension()? {
            return Ok(());
        }
        if !self.debug_tasks && self.task_id == 0 {
            loop {
                if let Err(error) = self.wait_observing_scopes() {
                    self.retire_wait_error(error)?;
                }
                if self.poll_task_suspension()? {
                    break;
                }
            }
        }
        Ok(())
    }

    pub(in crate::execution::vm) fn poll_acp_wait(&mut self, wait: Wait) -> Result<bool, VmError> {
        if let Wait::Native(wait) = wait {
            return self.poll_acp_native(wait);
        }
        let location = self.intrinsic_location();
        let (operation, destination, result) = match &wait {
            Wait::Native(_) => unreachable!("native ACP polling was handled above"),
            Wait::Receive {
                reader,
                token,
                deadline,
                destination,
            } => {
                let cancelled = token
                    .map(|token| {
                        self.hosted
                            .cancellations
                            .is_cancelled(token)
                            .map_err(|_| Fault::InvalidArgument("CancellationToken"))
                    })
                    .transpose();
                let result = match cancelled {
                    Err(error) => Some(Err(error)),
                    Ok(Some(true)) => Some(Err(Fault::Cancelled)),
                    _ => match reader.try_receive() {
                        Ok(Some(event)) => {
                            return self.finish_acp_wait(
                                *destination,
                                Ok(self.acp_event(event, location)?),
                                "Receive",
                                location,
                            );
                        }
                        Err(error) => Some(Err(error)),
                        Ok(None)
                            if deadline.is_some_and(|deadline| {
                                self.task_clock_ref().now_millis() >= deadline
                            }) =>
                        {
                            Some(Err(Fault::Timeout))
                        }
                        Ok(None) => None,
                    },
                };
                ("Receive", *destination, result)
            }
            Wait::Close {
                connection,
                first,
                release,
                destination,
            } => {
                let result = if connection.is_closed() {
                    self.hosted.mcp.release_owned(connection.id());
                    Some(connection.cleanup_result().and_then(|()| {
                        if *release {
                            self.hosted.acp.release(connection).map(Value::Boolean)
                        } else {
                            Ok(Value::Boolean(*first))
                        }
                    }))
                } else {
                    None
                };
                (
                    if *release { "Release" } else { "Close" },
                    *destination,
                    result,
                )
            }
        };
        if let Some(result) = result {
            self.finish_acp_wait(destination, result, operation, location)
        } else {
            self.task_suspension = Some(TaskSuspension::Acp(Box::new(wait)));
            Ok(false)
        }
    }

    fn finish_acp_wait(
        &mut self,
        destination: Option<Register>,
        result: Result<Value, Fault>,
        operation: &str,
        location: SourceLocation,
    ) -> Result<bool, VmError> {
        let value = self.acp_result(operation, result, location)?;
        if let Some(destination) = destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }
}

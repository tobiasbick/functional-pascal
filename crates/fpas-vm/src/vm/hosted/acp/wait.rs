//! Cooperative receive/close ownership shared by pool tasks and ordinary debugger tasks.

use super::arguments as args;
use crate::vm::{
    VmError,
    tasks::{TaskClock, TaskSuspension, TaskSuspensionState},
    worker::Worker,
};
use fpas_acp::connection::{Connection, Fault, Reader};
use fpas_bytecode::{AcpIntrinsic as Op, Intrinsic, Register, SourceLocation, Value};
use std::time::Duration;

pub(in crate::vm) enum Wait {
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

impl Wait {
    pub(in crate::vm) fn debug_state(&self, clock: &TaskClock) -> TaskSuspensionState {
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
    pub(in crate::vm) fn acp_wait_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Acp(operation) = intrinsic else {
            return Ok(None);
        };
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
        // The main lane uses the same saved suspension path as tasks and the debugger.
        self.task_suspension = Some(TaskSuspension::Acp(Box::new(wait)));
        self.suspend_requested = true;
        if self.poll_task_suspension()? {
            return Ok(Some(None));
        }
        if !self.debug_tasks && self.task_id == 0 {
            loop {
                self.wait_observing_scopes()?;
                if self.poll_task_suspension()? {
                    break;
                }
            }
        }
        Ok(Some(None))
    }

    pub(in crate::vm) fn poll_acp_wait(&mut self, wait: Wait) -> Result<bool, VmError> {
        let location = self.intrinsic_location();
        let (operation, destination, result) = match &wait {
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

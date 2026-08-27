//! Hosted `Std.Net` intrinsic dispatch.

mod connections;
mod listeners;
mod tls;
mod transport;

pub(super) use connections::NetworkConnections;
pub(super) use listeners::NetworkListeners;

use fpas_bytecode::{Intrinsic, NetIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::VmError;
use super::super::worker::Worker;

impl Worker {
    pub(super) fn execute_net_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Net(operation) = intrinsic else {
            return Ok(None);
        };
        let value = match operation {
            NetIntrinsic::Connect => {
                require_count(self, arguments, 3)?;
                let host = string(self, &arguments[0], "Host")?;
                let port = integer(self, &arguments[1], "Port")?;
                let timeout = integer(self, &arguments[2], "TimeoutMillis")?;
                result(
                    self.hosted
                        .network_connections
                        .connect_tcp(host, port, timeout)
                        .map(Value::OpaqueHandle),
                )
            }
            NetIntrinsic::ConnectTls => {
                require_count(self, arguments, 3)?;
                let host = string(self, &arguments[0], "Host")?;
                let port = integer(self, &arguments[1], "Port")?;
                let timeout = integer(self, &arguments[2], "TimeoutMillis")?;
                result(
                    self.hosted
                        .network_connections
                        .connect_tls(host, port, timeout)
                        .map(Value::OpaqueHandle),
                )
            }
            NetIntrinsic::Listen => {
                require_count(self, arguments, 2)?;
                let host = string(self, &arguments[0], "Host")?;
                let port = integer(self, &arguments[1], "Port")?;
                result(
                    self.hosted
                        .network_listeners
                        .listen(host, port)
                        .map(Value::OpaqueHandle),
                )
            }
            NetIntrinsic::ListenTls => {
                require_count(self, arguments, 5)?;
                let host = string(self, &arguments[0], "Host")?;
                let port = integer(self, &arguments[1], "Port")?;
                let certificate_path = string(self, &arguments[2], "CertificatePath")?;
                let private_key_path = string(self, &arguments[3], "PrivateKeyPath")?;
                let timeout = integer(self, &arguments[4], "HandshakeTimeoutMillis")?;
                result(
                    self.hosted
                        .network_listeners
                        .listen_tls(host, port, certificate_path, private_key_path, timeout)
                        .map(Value::OpaqueHandle),
                )
            }
            NetIntrinsic::Accept => {
                require_count(self, arguments, 1)?;
                let handle = listener(self, &arguments[0])?;
                result(
                    self.hosted
                        .network_listeners
                        .accept(handle)
                        .and_then(|transport| {
                            self.hosted.network_connections.insert_accepted(transport)
                        })
                        .map(Value::OpaqueHandle),
                )
            }
            NetIntrinsic::CloseListener => {
                require_count(self, arguments, 1)?;
                let handle = listener(self, &arguments[0])?;
                result(
                    self.hosted
                        .network_listeners
                        .close(handle)
                        .map(|()| Value::Boolean(true)),
                )
            }
            NetIntrinsic::SetTimeout => {
                require_count(self, arguments, 2)?;
                let handle = connection(self, &arguments[0])?;
                let timeout = integer(self, &arguments[1], "TimeoutMillis")?;
                result(
                    self.hosted
                        .network_connections
                        .set_timeout(handle, timeout)
                        .map(|()| Value::Boolean(true)),
                )
            }
            NetIntrinsic::Read => {
                require_count(self, arguments, 2)?;
                let handle = connection(self, &arguments[0])?;
                let max_bytes = integer(self, &arguments[1], "MaxBytes")?;
                result(
                    self.hosted
                        .network_connections
                        .read(handle, max_bytes)
                        .map(|bytes| {
                            Value::Array(
                                bytes
                                    .into_iter()
                                    .map(|byte| Value::Integer(i64::from(byte)))
                                    .collect(),
                            )
                        }),
                )
            }
            NetIntrinsic::Write => {
                require_count(self, arguments, 2)?;
                let handle = connection(self, &arguments[0])?;
                let bytes = bytes(self, &arguments[1])?;
                result(
                    self.hosted
                        .network_connections
                        .write(handle, &bytes)
                        .and_then(|count| {
                            i64::try_from(count).map(Value::Integer).map_err(|_| {
                                "TCP write count exceeds FPAS integer range".to_string()
                            })
                        }),
                )
            }
            NetIntrinsic::Close => {
                require_count(self, arguments, 1)?;
                let handle = connection(self, &arguments[0])?;
                result(
                    self.hosted
                        .network_connections
                        .close(handle)
                        .map(|()| Value::Boolean(true)),
                )
            }
        };
        Ok(Some(Some(value)))
    }
}

fn result(value: Result<Value, String>) -> Value {
    match value {
        Ok(value) => Value::ResultOk(Box::new(value)),
        Err(message) => Value::ResultError(Box::new(Value::Str(message.into()))),
    }
}

fn require_count(worker: &Worker, arguments: &[Value], expected: usize) -> Result<(), VmError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(worker.runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        format!(
            "Std.Net intrinsic expected {expected} arguments, got {}",
            arguments.len()
        ),
        "Check the compiler intrinsic signature and register argument count.",
    ))
}

fn string<'a>(worker: &Worker, value: &'a Value, name: &str) -> Result<&'a str, VmError> {
    match value {
        Value::Str(value) => Ok(value),
        actual => Err(type_error(worker, name, "string", actual)),
    }
}

fn integer(worker: &Worker, value: &Value, name: &str) -> Result<i64, VmError> {
    match value {
        Value::Integer(value) => Ok(*value),
        actual => Err(type_error(worker, name, "integer", actual)),
    }
}

fn connection(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(
            worker,
            "Connection",
            "Std.Net.Connection",
            actual,
        )),
    }
}

fn listener(worker: &Worker, value: &Value) -> Result<u64, VmError> {
    match value {
        Value::OpaqueHandle(handle) => Ok(*handle),
        actual => Err(type_error(worker, "Listener", "Std.Net.Listener", actual)),
    }
}

fn bytes(worker: &Worker, value: &Value) -> Result<Vec<u8>, VmError> {
    let Value::Array(values) = value else {
        return Err(type_error(worker, "Data", "array of integer", value));
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Value::Integer(value) => u8::try_from(*value).map_err(|_| {
                worker.runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Std.Net.Write Data[{index}] must be in 0..=255, got {value}"),
                    "Pass a byte array whose integer elements are in 0..=255.",
                )
            }),
            actual => Err(type_error(worker, "Data element", "integer", actual)),
        })
        .collect()
}

fn type_error(worker: &Worker, name: &str, expected: &str, actual: &Value) -> VmError {
    worker.runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Net {name} expected {expected}, got {}",
            actual.type_name()
        ),
        "Pass values matching the documented Std.Net function signature.",
    )
}

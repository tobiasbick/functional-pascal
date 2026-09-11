//! Hosted `Std.Net` intrinsic dispatch.

mod arguments;
mod connections;
mod listeners;
mod tls;
mod transport;

pub(super) use connections::NetworkConnections;
pub(super) use listeners::NetworkListeners;

use arguments::{bytes, cancellation_token, connection, integer, listener, require_count, string};
use fpas_bytecode::{Intrinsic, NetIntrinsic, SourceLocation, Value};

use super::super::VmError;
use super::super::worker::Worker;

impl Worker {
    /// Execute hosted network operations described in `docs/pascal/std/network/net.md`.
    pub(super) fn execute_net_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Net(operation) = intrinsic else {
            return Ok(None);
        };
        let value = match operation {
            NetIntrinsic::ConnectWithCancellation | NetIntrinsic::ConnectTlsWithCancellation => {
                require_count(self, arguments, 4)?;
                let host = string(self, &arguments[0], "Host")?;
                let port = integer(self, &arguments[1], "Port")?;
                let timeout = integer(self, &arguments[2], "TimeoutMillis")?;
                let token = cancellation_token(self, &arguments[3])?;
                let mode = if operation == NetIntrinsic::ConnectWithCancellation {
                    connections::ConnectMode::Tcp
                } else {
                    connections::ConnectMode::Tls
                };
                result(
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .and_then(|_| {
                            self.hosted.network_connections.connect_with_cancellation(
                                host,
                                port,
                                timeout,
                                mode,
                                || {
                                    self.hosted
                                        .cancellations
                                        .is_cancelled(token)
                                        .unwrap_or(true)
                                },
                            )
                        })
                        .map(Value::OpaqueHandle),
                )
            }
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
            NetIntrinsic::ListenerLocalAddress => {
                require_count(self, arguments, 1)?;
                let handle = listener(self, &arguments[0])?;
                match self.hosted.network_listeners.local_address(handle) {
                    Ok(address) => Value::result_ok(self.record_value(
                        "Std.Net.NetworkAddress",
                        vec![
                            Value::Str(address.ip().to_string().into()),
                            Value::Integer(i64::from(address.port())),
                        ],
                        location,
                    )?),
                    Err(message) => result(Err(message)),
                }
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
            NetIntrinsic::AcceptWithCancellation => {
                require_count(self, arguments, 2)?;
                let handle = listener(self, &arguments[0])?;
                let token = cancellation_token(self, &arguments[1])?;
                result(
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .and_then(|cancelled| {
                            if cancelled {
                                return Err("Network accept cancelled".to_string());
                            }
                            self.hosted
                                .network_listeners
                                .accept_with_cancellation(handle, || {
                                    self.hosted
                                        .cancellations
                                        .is_cancelled(token)
                                        .unwrap_or(true)
                                })
                        })
                        .and_then(|transport| {
                            if self
                                .hosted
                                .cancellations
                                .is_cancelled(token)
                                .unwrap_or(true)
                            {
                                Err("Network accept cancelled".to_string())
                            } else {
                                self.hosted.network_connections.insert_accepted(transport)
                            }
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
            NetIntrinsic::Read | NetIntrinsic::ReadWithCancellation => {
                let cancellable = operation == NetIntrinsic::ReadWithCancellation;
                require_count(self, arguments, if cancellable { 3 } else { 2 })?;
                let handle = connection(self, &arguments[0])?;
                let max_bytes = integer(self, &arguments[1], "MaxBytes")?;
                let read = if cancellable {
                    let token = cancellation_token(self, &arguments[2])?;
                    self.hosted.cancellations.is_cancelled(token).and_then(|_| {
                        self.hosted.network_connections.read_with_cancellation(
                            handle,
                            max_bytes,
                            || {
                                self.hosted
                                    .cancellations
                                    .is_cancelled(token)
                                    .unwrap_or(true)
                            },
                        )
                    })
                } else {
                    self.hosted.network_connections.read(handle, max_bytes)
                };
                result(read.map(|bytes| {
                    Value::Array(
                        bytes
                            .into_iter()
                            .map(|byte| Value::Integer(i64::from(byte)))
                            .collect(),
                    )
                }))
            }
            NetIntrinsic::Write | NetIntrinsic::WriteWithCancellation => {
                let cancellable = operation == NetIntrinsic::WriteWithCancellation;
                require_count(self, arguments, if cancellable { 3 } else { 2 })?;
                let handle = connection(self, &arguments[0])?;
                let bytes = bytes(self, &arguments[1])?;
                let write = if cancellable {
                    let token = cancellation_token(self, &arguments[2])?;
                    self.hosted.cancellations.is_cancelled(token).and_then(|_| {
                        self.hosted.network_connections.write_with_cancellation(
                            handle,
                            &bytes,
                            || {
                                self.hosted
                                    .cancellations
                                    .is_cancelled(token)
                                    .unwrap_or(true)
                            },
                        )
                    })
                } else {
                    self.hosted.network_connections.write(handle, &bytes)
                };
                result(write.and_then(|count| {
                    i64::try_from(count)
                        .map(Value::Integer)
                        .map_err(|_| "TCP write count exceeds FPAS integer range".to_string())
                }))
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
        Ok(value) => Value::result_ok(value),
        Err(message) => Value::result_error(Value::Str(message.into())),
    }
}

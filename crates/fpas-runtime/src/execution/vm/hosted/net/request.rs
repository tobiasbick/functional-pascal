//! Owned socket requests; FPAS arguments and result construction stay on the VM thread.

use super::connections::{ConnectMode, ConnectRequest};
use super::{Failure, arguments as args};
use crate::execution::vm::{VmError, hosted::HostedState, worker::Worker};
use crate::{NetIntrinsic as Op, Value};

pub(super) enum Operation {
    Connect(ConnectRequest),
    Listen(String, i64),
    ListenTls {
        host: String,
        port: i64,
        certificate: String,
        key: String,
        timeout: i64,
    },
    Accept(u64),
    Read(u64, i64),
    Write(u64, Vec<u8>),
    SetTimeout(u64, i64),
}

pub(super) enum Output {
    Connection(u64),
    Listener(u64),
    Bytes(Vec<u8>),
    Count(usize),
    Done,
}

pub(super) struct Prepared {
    pub(super) operation: Result<Operation, Failure>,
    pub(super) token: Option<u64>,
    pub(super) resources: Vec<String>,
}

pub(super) fn prepare(worker: &Worker, op: Op, values: &[Value]) -> Result<Prepared, VmError> {
    let cancellable = matches!(
        op,
        Op::AcceptWithCancellation
            | Op::ReadWithCancellation
            | Op::WriteWithCancellation
            | Op::ConnectWithCancellation
            | Op::ConnectTlsWithCancellation
    );
    let count = match op {
        Op::Connect
        | Op::ConnectTls
        | Op::ConnectWithCancellation
        | Op::ConnectTlsWithCancellation => 3,
        Op::ListenTls => 5,
        Op::Accept | Op::AcceptWithCancellation => 1,
        _ => 2,
    };
    args::require_count(worker, values, count + usize::from(cancellable))?;
    let token = if cancellable {
        Some(args::cancellation_token(worker, &values[count])?)
    } else {
        None
    };
    let (operation, resources) = match op {
        Op::Connect
        | Op::ConnectTls
        | Op::ConnectWithCancellation
        | Op::ConnectTlsWithCancellation
        | Op::Listen
        | Op::ListenTls => {
            let host = args::string(worker, &values[0], "Host")?;
            let port = args::integer(worker, &values[1], "Port")?;
            let mut resources = vec![host.to_owned(), port.to_string()];
            let operation = match op {
                Op::Listen => Ok(Operation::Listen(host.to_owned(), port)),
                Op::ListenTls => {
                    let certificate = args::string(worker, &values[2], "CertificatePath")?;
                    let key = args::string(worker, &values[3], "PrivateKeyPath")?;
                    let timeout = args::integer(worker, &values[4], "HandshakeTimeoutMillis")?;
                    resources.extend([certificate.to_owned(), key.to_owned()]);
                    Ok(Operation::ListenTls {
                        host: host.to_owned(),
                        port,
                        certificate: certificate.to_owned(),
                        key: key.to_owned(),
                        timeout,
                    })
                }
                _ => {
                    let timeout = args::integer(worker, &values[2], "TimeoutMillis")?;
                    let mode = if matches!(op, Op::ConnectTls | Op::ConnectTlsWithCancellation) {
                        ConnectMode::Tls
                    } else {
                        ConnectMode::Tcp
                    };
                    ConnectRequest::new(host, port, timeout, mode).map(Operation::Connect)
                }
            };
            (operation, resources)
        }
        _ => {
            let accept = matches!(op, Op::Accept | Op::AcceptWithCancellation);
            let handle = if accept {
                args::listener(worker, &values[0])?
            } else {
                args::connection(worker, &values[0])?
            };
            let operation = match op {
                Op::Accept | Op::AcceptWithCancellation => Ok(Operation::Accept(handle)),
                Op::Read | Op::ReadWithCancellation => Ok(Operation::Read(
                    handle,
                    args::integer(worker, &values[1], "MaxBytes")?,
                )),
                Op::Write | Op::WriteWithCancellation => {
                    args::bytes(worker, &values[1])?.map(|bytes| Operation::Write(handle, bytes))
                }
                Op::SetTimeout => Ok(Operation::SetTimeout(
                    handle,
                    args::integer(worker, &values[1], "TimeoutMillis")?,
                )),
                _ => unreachable!("only suspending socket operations are prepared"),
            };
            (operation, vec![handle.to_string()])
        }
    };
    Ok(Prepared {
        operation,
        token,
        resources,
    })
}

impl Operation {
    pub(super) fn run(
        self,
        state: &HostedState,
        cancelled: impl Fn() -> bool,
    ) -> Result<Output, Failure> {
        match self {
            Self::Connect(request) => state
                .network_connections
                .connect_prepared(&request, cancelled)
                .map(Output::Connection),
            Self::Listen(host, port) => state
                .network_listeners
                .listen_with_cancellation(&host, port, cancelled)
                .map(Output::Listener),
            Self::ListenTls {
                host,
                port,
                certificate,
                key,
                timeout,
            } => state
                .network_listeners
                .listen_tls_with_cancellation(&host, port, &certificate, &key, timeout, cancelled)
                .map(Output::Listener),
            Self::Accept(handle) => state
                .network_listeners
                .accept_with_cancellation(handle, &cancelled)
                .and_then(|transport| {
                    if cancelled() {
                        Err(Failure::Cancelled)
                    } else {
                        state
                            .network_connections
                            .insert_accepted(transport)
                            .map(Output::Connection)
                    }
                }),
            Self::Read(handle, count) => state
                .network_connections
                .read_with_cancellation(handle, count, cancelled)
                .map(Output::Bytes),
            Self::Write(handle, bytes) => state
                .network_connections
                .write_with_cancellation(handle, &bytes, cancelled)
                .map(Output::Count),
            Self::SetTimeout(handle, timeout) => state
                .network_connections
                .set_timeout_with_cancellation(handle, timeout, cancelled)
                .map(|()| Output::Done),
        }
    }
}

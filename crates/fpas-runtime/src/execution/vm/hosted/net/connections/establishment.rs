//! Cooperative connection establishment with native DNS and cancellable TCP/TLS I/O.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use crate::execution::vm::hosted::net::failure::Failure;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::super::tls::client;
use super::{NetworkConnections, Transport, connect_port, timeout};

mod resolver;
mod tcp;

/// Protocol to establish before publishing a connection handle.
#[derive(Clone, Copy)]
pub(in crate::execution::vm::hosted::net) enum ConnectMode {
    /// Plain TCP connection.
    Tcp,
    /// TCP followed by a verified TLS handshake.
    Tls,
}

/// Validated native data; the deadline starts before worker admission.
pub(in crate::execution::vm::hosted::net) struct ConnectRequest {
    host: String,
    port: u16,
    deadline: Instant,
    mode: ConnectMode,
}

impl ConnectRequest {
    pub(in crate::execution::vm::hosted::net) fn new(
        host: &str,
        port: i64,
        timeout_millis: i64,
        mode: ConnectMode,
    ) -> Result<Self, Failure> {
        let port = connect_port(port)?;
        let deadline = Instant::now() + timeout(timeout_millis, false)?;
        Ok(Self {
            host: host.to_owned(),
            port,
            deadline,
            mode,
        })
    }
}

impl NetworkConnections {
    /// Establish a connection without publishing cancelled or expired attempts.
    ///
    /// Resolution waits and TCP/TLS attempts share one cancellation-aware deadline.
    pub(in crate::execution::vm::hosted::net) fn connect_with_cancellation(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
        mode: ConnectMode,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<u64, Failure> {
        let request = ConnectRequest::new(host, port, timeout_millis, mode)?;
        self.connect_prepared(&request, is_cancelled)
    }

    pub(in crate::execution::vm::hosted::net) fn connect_tls(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
    ) -> Result<u64, Failure> {
        self.connect_with_cancellation(host, port, timeout_millis, ConnectMode::Tls, || false)
    }

    pub(in crate::execution::vm::hosted::net) fn connect_prepared(
        &self,
        request: &ConnectRequest,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<u64, Failure> {
        let ConnectRequest {
            host,
            port,
            deadline,
            mode,
        } = request;
        let deadline = *deadline;
        let cancelled = || is_cancelled() || self.shutdown.load(Ordering::Acquire);
        remaining(deadline, cancelled)?;
        let addresses = resolver::resolve(host, *port, deadline, cancelled)?;
        remaining(deadline, cancelled)?;
        let mut last_error = None;
        for address in addresses {
            let connected = tcp::connect(address, deadline, cancelled);
            remaining(deadline, cancelled)?;
            let stream = match connected {
                Ok(stream) => stream,
                Err(error) => {
                    last_error = Some(error);
                    continue;
                }
            };
            stream.set_nodelay(true).map_err(|error| {
                Failure::io(
                    &error,
                    format!("Could not configure TCP connection: {error}"),
                )
            })?;
            let transport = match mode {
                ConnectMode::Tcp => Transport::tcp(stream),
                ConnectMode::Tls => {
                    let connected = client::connect(stream, host, deadline, cancelled);
                    remaining(deadline, cancelled)?;
                    Transport::tls_client(connected?)
                }
            };
            remaining(deadline, cancelled)?;
            return self.insert(transport);
        }
        Err(last_error.unwrap_or_else(|| {
            Failure::no_addresses(format!("TCP host '{host}' resolved to no addresses"))
        }))
    }
}

fn remaining(deadline: Instant, is_cancelled: impl Fn() -> bool) -> Result<Duration, Failure> {
    if is_cancelled() {
        return Err(Failure::Cancelled);
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| Failure::deadline("Network connect timed out"))
}

#[cfg(test)]
mod tests;

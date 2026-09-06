//! Cooperative connection establishment with native DNS and cancellable TCP/TLS I/O.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use std::net::ToSocketAddrs;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::super::tls::client;
use super::{NetworkConnections, Transport, connect_port, timeout};

mod tcp;

/// Protocol to establish before publishing a connection handle.
#[derive(Clone, Copy)]
pub(in crate::vm::hosted::net) enum ConnectMode {
    /// Plain TCP connection.
    Tcp,
    /// TCP followed by a verified TLS handshake.
    Tls,
}

impl NetworkConnections {
    /// Establish a connection without publishing cancelled or expired attempts.
    ///
    /// OS resolution observes cancellation on return; TCP attempts poll the token while pending.
    pub(in crate::vm::hosted::net) fn connect_with_cancellation(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
        mode: ConnectMode,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<u64, String> {
        let port = connect_port(port)?;
        let deadline = Instant::now() + timeout(timeout_millis, false)?;
        let cancelled = || is_cancelled() || self.shutdown.load(Ordering::Acquire);
        remaining(deadline, cancelled)?;
        let addresses = (host, port).to_socket_addrs();
        remaining(deadline, cancelled)?;
        let addresses =
            addresses.map_err(|error| format!("Could not resolve TCP host '{host}': {error}"))?;
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
            stream
                .set_nodelay(true)
                .map_err(|error| format!("Could not configure TCP connection: {error}"))?;
            let transport = match mode {
                ConnectMode::Tcp => Transport::tcp(stream),
                ConnectMode::Tls => {
                    let connected =
                        client::connect(stream, host, remaining(deadline, cancelled)?, cancelled);
                    remaining(deadline, cancelled)?;
                    Transport::tls_client(connected?)
                }
            };
            remaining(deadline, cancelled)?;
            return self.insert(transport);
        }
        Err(last_error.map_or_else(
            || format!("TCP host '{host}' resolved to no addresses"),
            |error| format!("Could not connect to {host}:{port}: {error}"),
        ))
    }
}

fn remaining(deadline: Instant, is_cancelled: impl Fn() -> bool) -> Result<Duration, String> {
    if is_cancelled() {
        return Err("Network connect cancelled".to_string());
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| "Network connect timed out".to_string())
}

#[cfg(test)]
mod tests;

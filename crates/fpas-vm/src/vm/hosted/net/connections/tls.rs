//! Cancellation-aware outgoing TLS setup.
//!
//! Documentation: `docs/pascal/std/network/net.md`.

use std::sync::atomic::Ordering;

use super::super::{tls::client, transport::Transport};
use super::{NetworkConnections, connect_socket};

impl NetworkConnections {
    /// Complete a verified TLS connection while observing VM cancellation.
    pub(in crate::vm::hosted::net) fn connect_tls(
        &self,
        host: &str,
        port: i64,
        timeout_millis: i64,
    ) -> Result<u64, String> {
        let (stream, timeout) = connect_socket(host, port, timeout_millis)?;
        let stream = client::connect(stream, host, timeout, || {
            self.shutdown.load(Ordering::Acquire)
        })?;
        self.insert(Transport::tls_client(stream))
    }
}

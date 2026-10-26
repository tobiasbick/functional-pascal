//! Loopback listener bounds, independent of application-call admission.

use crate::Fault;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

/// Resource and timeout policy for one authenticated HTTP attachment.
#[derive(Clone, Debug)]
pub struct HttpOptions {
    /// Loopback address; port zero requests an unused operating-system port.
    pub address: SocketAddr,
    /// Maximum accepted sockets, including incomplete HTTP requests.
    pub connections: usize,
    /// Deadline for one connection and its application request.
    pub request_timeout: Duration,
    /// Maximum time to drain connections after retiring the attachment.
    pub shutdown_timeout: Duration,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            connections: 64,
            request_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

impl HttpOptions {
    pub(super) fn validate(&self) -> Result<(), Fault> {
        if !self.address.ip().is_loopback() {
            return Err(Fault::InvalidArgument("Address"));
        }
        if self.connections == 0 || self.connections > 4096 {
            return Err(Fault::InvalidArgument("Connections"));
        }
        for (name, value, maximum) in [
            (
                "RequestTimeout",
                self.request_timeout,
                Duration::from_secs(300),
            ),
            (
                "ShutdownTimeout",
                self.shutdown_timeout,
                Duration::from_secs(30),
            ),
        ] {
            if value.is_zero() || value > maximum {
                return Err(Fault::InvalidArgument(name));
            }
        }
        Ok(())
    }
}

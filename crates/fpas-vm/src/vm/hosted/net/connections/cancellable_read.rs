//! Cooperative reads that preserve TCP/TLS state and socket timeout configuration.

use std::io::Read;

use super::polling::Direction;
use super::{NetworkConnections, read_size};

impl NetworkConnections {
    /// Read a bounded chunk without closing the connection on cancellation.
    ///
    /// **Documentation:** `docs/pascal/std/network/net.md`.
    pub(in crate::vm::hosted::net) fn read_with_cancellation(
        &self,
        handle: u64,
        max_bytes: i64,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Vec<u8>, String> {
        let mut bytes = vec![0; read_size(max_bytes)?];
        let count = self.poll_io(handle, Direction::Read, is_cancelled, |transport| {
            transport.read(&mut bytes)
        })?;
        bytes.truncate(count);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests;

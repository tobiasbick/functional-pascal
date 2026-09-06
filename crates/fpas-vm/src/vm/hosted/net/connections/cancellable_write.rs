//! Cancellation-aware writes with explicit partial-progress reporting.

use std::io::Write;

use super::polling::Direction;
use super::{NetworkConnections, write_size};

impl NetworkConnections {
    /// Write one chunk, returning accepted bytes even if cancellation races with progress.
    ///
    /// **Documentation:** `docs/pascal/std/network/net.md`.
    pub(in crate::vm::hosted::net) fn write_with_cancellation(
        &self,
        handle: u64,
        bytes: &[u8],
        is_cancelled: impl Fn() -> bool,
    ) -> Result<usize, String> {
        write_size(bytes.len())?;
        self.poll_io(handle, Direction::Write, is_cancelled, |transport| {
            transport.write(bytes)
        })
    }
}

#[cfg(test)]
mod tests;

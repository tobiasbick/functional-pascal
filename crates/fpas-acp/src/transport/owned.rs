//! Transport ownership shared with host process adapters, without host dependencies.

use crate::{RawJson, connection::Fault};
use futures::future::BoxFuture;

/// A complete input frame retaining its admission credit through SDK dispatch.
pub struct Frame {
    pub(crate) raw: RawJson,
    _credit: Option<Box<dyn Send>>,
}

impl Frame {
    /// A frame whose storage is owned by the host adapter.
    pub fn new(raw: RawJson) -> Self {
        Self { raw, _credit: None }
    }

    pub(crate) fn credited(raw: RawJson, credit: impl Send + 'static) -> Self {
        Self {
            raw,
            _credit: Some(Box::new(credit)),
        }
    }
}

/// Precise process termination, independent of platform-specific status types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    /// Normal process exit with its complete unsigned status.
    Exited(u32),
    /// POSIX termination signal.
    Signaled(i32),
    /// A platform did not provide either status form.
    Unknown,
}

/// Detached transport cleanup result; stderr remains separate from protocol data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportStatus {
    /// Present only for a subprocess transport.
    pub exit: Option<ExitStatus>,
    /// Bounded retained stderr, decoded with replacement for invalid bytes.
    pub stderr: String,
    /// Additional stderr was drained and discarded.
    pub stderr_truncated: bool,
}

/// Owned physical transport driven exclusively by the connection executor.
/// Methods must yield rather than sleep when native I/O is pending.
pub trait Transport: Send + Sync + 'static {
    /// Read one frame, or EOF after all previously admitted frames.
    fn receive(&self) -> BoxFuture<'_, Result<Option<Frame>, Fault>>;
    /// Write one complete frame, preserving partial-write ownership until completion.
    fn send(&self, frame: RawJson) -> BoxFuture<'_, Result<(), Fault>>;
    /// Close input, allow a bounded grace, then terminate and reap owned resources.
    fn finish(&self, grace_millis: u64) -> BoxFuture<'_, Result<TransportStatus, Fault>>;
}

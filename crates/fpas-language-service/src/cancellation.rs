//! Cooperative cancellation for bounded language-service work.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Cloneable signal checked by manifest discovery and long navigation scans.
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a signal that has not been cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation for every clone of this signal.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Returns a cancellation error after cancellation was requested.
    pub fn check(&self) -> Result<(), crate::LanguageServiceError> {
        if self.is_cancelled() {
            Err(crate::LanguageServiceError::Cancelled)
        } else {
            Ok(())
        }
    }
}

//! Opaque cancellation-source and token registry.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::vm::shared::wakeups::{WakeRegistration, WakeSignal, WakeSource};

const HANDLE_TAG: u64 = 0x4341_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    changed: Arc<WakeSource>,
}

type Entries = Mutex<HashMap<u64, Arc<CancellationState>>>;

/// A resource-owned source whose token storage is released with its owner.
pub(in crate::vm) struct OwnedCancellation {
    handle: u64,
    state: Arc<CancellationState>,
    entries: Weak<Entries>,
}

impl OwnedCancellation {
    /// Return the opaque token identity while its owner is alive.
    pub(in crate::vm) fn token(&self) -> u64 {
        self.handle
    }

    /// Request cancellation, reporting whether this changed the source.
    pub(in crate::vm) fn cancel(&self) -> bool {
        self.state.cancel()
    }

    /// Inspect admission cancellation without looking up the token again.
    pub(in crate::vm) fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }
}

impl Drop for OwnedCancellation {
    fn drop(&mut self) {
        self.cancel();
        if let Some(entries) = self.entries.upgrade() {
            entries
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&self.handle);
        }
    }
}

impl CancellationState {
    fn cancel(&self) -> bool {
        let changed = !self.cancelled.swap(true, Ordering::AcqRel);
        if changed {
            self.changed.notify();
        }
        changed
    }
}

/// Cancellation state owned by one VM and shared by all of its tasks.
pub(in crate::vm) struct CancellationRegistry {
    next_handle: AtomicU64,
    entries: Arc<Entries>,
}

impl CancellationRegistry {
    /// Create an empty per-VM cancellation registry.
    pub(in crate::vm) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            entries: Arc::default(),
        }
    }

    /// Create a cancellation source and return its opaque handle.
    pub(in crate::vm) fn create_source(&self) -> u64 {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(handle, Arc::default());
        handle
    }

    /// Create a token whose identity and state live only as long as its resource owner.
    pub(in crate::vm) fn create_owned(&self) -> OwnedCancellation {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        let state = Arc::new(CancellationState::default());
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(handle, Arc::clone(&state));
        OwnedCancellation {
            handle,
            state,
            entries: Arc::downgrade(&self.entries),
        }
    }

    /// Validate a source and return the same identity as its clonable token.
    pub(in crate::vm) fn token(&self, source: u64) -> Result<u64, String> {
        self.state(source)?;
        Ok(source)
    }

    /// Request cancellation and report whether this call changed the state.
    pub(in crate::vm) fn cancel(&self, source: u64) -> Result<bool, String> {
        Ok(self.state(source)?.cancel())
    }

    /// Return whether cancellation was requested for a token.
    pub(in crate::vm) fn is_cancelled(&self, token: u64) -> Result<bool, String> {
        Ok(self.state(token)?.cancelled.load(Ordering::Acquire))
    }

    /// Subscribe before inspecting cancellation; dropping the guard unregisters this wait.
    pub(in crate::vm) fn subscribe(
        &self,
        token: u64,
        signal: &Arc<WakeSignal>,
    ) -> Result<WakeRegistration, String> {
        Ok(self.state(token)?.changed.subscribe(signal))
    }

    fn state(&self, handle: u64) -> Result<Arc<CancellationState>, String> {
        validate_handle(handle)?;
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or_else(|| "Cancellation handle does not belong to this VM".to_string())
    }
}

fn validate_handle(handle: u64) -> Result<(), String> {
    if handle & HANDLE_TAG_MASK == HANDLE_TAG {
        Ok(())
    } else {
        Err("Value is not a cancellation handle".to_string())
    }
}

#[cfg(test)]
mod wakeup_tests;

#[cfg(test)]
mod tests {
    use super::CancellationRegistry;

    #[test]
    fn cancel_changes_one_shared_token_state_once() {
        let registry = CancellationRegistry::new();
        let source = registry.create_source();
        let token = registry.token(source).expect("token");

        assert_eq!(
            (
                registry.is_cancelled(token).expect("initial state"),
                registry.cancel(source).expect("first cancel"),
                registry.cancel(source).expect("second cancel"),
                registry.is_cancelled(token).expect("cancelled state"),
            ),
            (false, true, false, true)
        );
    }

    #[test]
    fn foreign_handle_is_rejected() {
        let registry = CancellationRegistry::new();

        assert_eq!(
            registry.is_cancelled(1).expect_err("foreign handle"),
            "Value is not a cancellation handle"
        );
    }
}

//! Wait-owned wake registrations shared by runtime event sources.
//!
//! A notification is only a request to inspect a source again, never ownership of its value.
//! Register before inspecting the source predicate. Notification locks are acquired in source,
//! then signal order; waiting holds only the signal lock and never executes task code.

use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

/// A latched notification that can be signalled by multiple event sources.
pub(in crate::vm) struct WakeSignal {
    notified: Mutex<bool>,
    changed: Condvar,
}

impl WakeSignal {
    /// Create a new, unnotified wait signal.
    pub(in crate::vm) fn new() -> Arc<Self> {
        Arc::new(Self {
            notified: Mutex::new(false),
            changed: Condvar::new(),
        })
    }

    /// Wait within one budget, retaining notifications published before parking.
    ///
    /// Returns true when notified. Repeated waits remain ready; create a new signal for a new
    /// observation cycle and register it before inspecting the source again.
    pub(in crate::vm) fn wait(&self, timeout: Duration) -> bool {
        let notified = self
            .notified
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (notified, _) = self
            .changed
            .wait_timeout_while(notified, timeout, |notified| !*notified)
            .unwrap_or_else(|error| error.into_inner());
        *notified
    }

    fn notify(&self) {
        *self
            .notified
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct Subscription {
    signal: Arc<WakeSignal>,
}

/// An event source's weak subscriptions; it does not own any waiting operation.
#[derive(Default)]
pub(in crate::vm) struct WakeSource {
    subscriptions: Mutex<Vec<Weak<Subscription>>>,
}

impl WakeSource {
    /// Inspect retained registrations in lifetime regression tests.
    #[cfg(test)]
    pub(in crate::vm) fn registration_count(&self) -> usize {
        self.subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    /// Register a signal, returning the sole owner of this subscription.
    pub(in crate::vm) fn subscribe(self: &Arc<Self>, signal: &Arc<WakeSignal>) -> WakeRegistration {
        let subscription = Arc::new(Subscription {
            signal: Arc::clone(signal),
        });
        self.subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(Arc::downgrade(&subscription));
        WakeRegistration {
            source: Arc::clone(self),
            subscription,
        }
    }

    /// Notify all current waits without calling user code or consuming source values.
    pub(in crate::vm) fn notify(&self) {
        self.subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .retain(|subscription| {
                if let Some(subscription) = subscription.upgrade() {
                    subscription.signal.notify();
                    true
                } else {
                    false
                }
            });
    }
}

/// Removes precisely one source subscription on every exit path.
pub(in crate::vm) struct WakeRegistration {
    source: Arc<WakeSource>,
    subscription: Arc<Subscription>,
}

impl Drop for WakeRegistration {
    fn drop(&mut self) {
        let own = Arc::downgrade(&self.subscription);
        self.source
            .subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .retain(|subscription| !Weak::ptr_eq(subscription, &own));
    }
}

#[cfg(test)]
mod tests;

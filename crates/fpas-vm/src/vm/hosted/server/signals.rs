//! Explicit process-wide signal ownership for standalone hosts.

use super::state::Lifetime;
use std::sync::{Arc, Mutex, OnceLock, Weak};

static SUBSCRIBERS: Mutex<Vec<Weak<Lifetime>>> = Mutex::new(Vec::new());
static INSTALLED: OnceLock<Result<(), String>> = OnceLock::new();

/// Install the process adapter once and subscribe a lifetime without retaining it.
pub(super) fn observe(lifetime: &Arc<Lifetime>) -> Result<bool, String> {
    INSTALLED
        .get_or_init(|| {
            ctrlc::try_set_handler(dispatch)
                .map_err(|e| format!("Cannot own process termination signals: {e}"))
        })
        .clone()?;
    let mut subscribers = SUBSCRIBERS.lock().unwrap_or_else(|e| e.into_inner());
    subscribers.retain(|entry| entry.upgrade().is_some_and(|life| life.ready()));
    if subscribers
        .iter()
        .any(|entry| entry.ptr_eq(&Arc::downgrade(lifetime)))
    {
        return Ok(false);
    }
    if subscribers.len() >= 16 {
        return Err("At most 16 server lifetimes may observe process signals".into());
    }
    subscribers.push(Arc::downgrade(lifetime));
    Ok(true)
}

/// Deliver a host signal through the common idempotent stop path.
pub(super) fn dispatch() {
    let targets: Vec<_> = SUBSCRIBERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .filter_map(Weak::upgrade)
        .collect();
    for target in targets {
        target.request_stop();
    }
}

#[cfg(test)]
/// Subscribe a fixture without modifying the test process's signal handler.
pub(super) fn subscribe_for_test(lifetime: &Arc<Lifetime>) {
    SUBSCRIBERS.lock().unwrap().push(Arc::downgrade(lifetime));
}

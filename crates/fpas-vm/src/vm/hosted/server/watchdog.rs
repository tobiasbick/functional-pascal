//! Deadline escalation never runs on a VM worker or writes diagnostics.

use super::state::Lifetime;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

/// Start host-authorized process escalation independently of VM and output progress.
pub(super) fn start(lifetime: &Arc<Lifetime>) -> Result<(), String> {
    let lifetime = Arc::downgrade(lifetime);
    std::thread::Builder::new()
        .name("fpas-shutdown-deadline".into())
        .spawn(move || {
            while let Some(lifetime) = Weak::upgrade(&lifetime) {
                let state = lifetime.progress.lock().unwrap_or_else(|e| e.into_inner());
                if state.finished {
                    return;
                }
                if state
                    .deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
                {
                    // Do not use process::exit: exit hooks and stream flushing can block.
                    // No FPAS callbacks, destructors, or diagnostic writers run on this path.
                    std::process::abort();
                }
                drop(state);
                drop(lifetime);
                std::thread::sleep(Duration::from_millis(10));
            }
        })
        .map(|_| ())
        .map_err(|e| format!("Cannot start shutdown deadline monitor: {e}"))
}

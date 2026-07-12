//! Cooperative shutdown for aborting in-flight [`super::Vm::run`] calls.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md)

use std::sync::Arc;

use super::shared::SharedState;

/// Cloneable handle for requesting VM shutdown from another thread.
pub struct VmShutdownHandle {
    shared: Arc<SharedState>,
}

impl VmShutdownHandle {
    pub(crate) fn new(shared: Arc<SharedState>) -> Self {
        Self { shared }
    }

    /// Aborts spawned tasks and signals hosted run loops to exit cooperatively.
    pub fn request_cooperative_shutdown(&self) {
        self.shared.signal_runtime_failure();
    }
}

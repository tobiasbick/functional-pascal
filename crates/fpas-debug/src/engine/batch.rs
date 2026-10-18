//! Direct batch access to the owned session without a wire protocol.

use super::DebugEngine;

impl DebugEngine {
    /// Keeps completed globals available until the batch adapter has copied its snapshot.
    pub(crate) fn retain_for_batch(&mut self) {
        self.retain_terminal = true;
    }

    /// Borrows the stable engine-owned session during batch setup or capture.
    pub(crate) fn batch_session(&mut self) -> Option<&mut fpas_vm::DebugSession> {
        self.actor.session_mut()
    }
}

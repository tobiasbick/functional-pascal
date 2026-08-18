//! All-worker validation and atomic live-image pointer replacement.

use super::DebugTaskRuntime;
use crate::vm::debug::live_image::PreparedLiveImageCommit;

impl DebugTaskRuntime {
    /// Verify that every retained worker can be remapped before commit begins.
    pub(in crate::vm::debug) fn validates_live_image_commit(
        &self,
        commit: &PreparedLiveImageCommit,
    ) -> bool {
        self.tasks
            .values()
            .all(|slot| commit.validates(&slot.worker))
    }

    /// Apply one prevalidated image to every retained worker.
    pub(in crate::vm::debug) fn commit_live_image(&mut self, commit: &PreparedLiveImageCommit) {
        for slot in self.tasks.values_mut() {
            commit.apply(&mut slot.worker);
        }
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn workers_share_live_image(
        &self,
        executable: &std::sync::Arc<fpas_bytecode::VerifiedExecutable>,
    ) -> bool {
        self.tasks
            .values()
            .all(|slot| std::sync::Arc::ptr_eq(&slot.worker.executable, executable))
    }
}

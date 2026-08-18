//! Session mapping onto live-image compatibility classification.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::VerifiedExecutable;

use super::DebugSession;
use crate::vm::debug::live_image::{
    LiveImageClassification, classify_live_image as classify_images,
};

impl DebugSession {
    /// Classify a candidate executable against the live image without replacing it.
    ///
    /// Active stack functions come from every retained worker. The live
    /// `Arc<VerifiedExecutable>` is unchanged.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub fn classify_live_image(&self, candidate: &VerifiedExecutable) -> LiveImageClassification {
        classify_images(
            &self.executable,
            candidate,
            &self.runtime.active_function_ids(),
        )
    }

    /// Classify the live image against itself.
    ///
    /// Used by adapters that name accepted and rejected classes without a
    /// second compiled candidate.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub fn classify_current_live_image(&self) -> LiveImageClassification {
        self.classify_live_image(&self.executable)
    }
}

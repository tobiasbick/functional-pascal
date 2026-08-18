//! Session mapping onto live-image classification and reject-before-commit.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::VerifiedExecutable;

use super::DebugSession;
use crate::vm::debug::live_image::{
    LiveImageClassification, LiveImageReplaceResult, LiveImageUpdateClass,
    classify_live_image as classify_images,
};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

impl DebugSession {
    /// Borrow the live executable without replacing it.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    #[must_use]
    pub fn live_executable(&self) -> &VerifiedExecutable {
        self.executable.as_ref()
    }

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

    /// Reject an incompatible candidate before the live image can change.
    ///
    /// Accepted classes are not applied yet; `applied` stays false until
    /// versioned commit exists. The live `Arc<VerifiedExecutable>` is unchanged.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns [`DebugErrorKind::LiveImageIncompatible`] when the candidate is
    /// not in the proven accepted subset.
    pub fn replace_live_image(
        &mut self,
        candidate: &VerifiedExecutable,
    ) -> Result<LiveImageReplaceResult, DebugSessionError> {
        let classification = self.classify_live_image(candidate);
        if !classification.accepted {
            return Err(incompatible_image(classification.class));
        }
        Ok(LiveImageReplaceResult::from_classification(classification))
    }

    /// Run the replace gate against the current live executable.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns [`DebugErrorKind::LiveImageIncompatible`] when classification
    /// rejects the live image against itself, which does not occur for a
    /// consistent executable.
    pub fn replace_current_live_image(
        &mut self,
    ) -> Result<LiveImageReplaceResult, DebugSessionError> {
        let candidate = std::sync::Arc::clone(&self.executable);
        self.replace_live_image(candidate.as_ref())
    }
}

fn incompatible_image(class: LiveImageUpdateClass) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::LiveImageIncompatible,
        message: format!(
            "live-image update `{}` is incompatible and was rejected before commit",
            class.as_str()
        ),
        hint: "Keep the current launch-owned executable. Incompatible layouts, active bodies, captures, tasks, and metadata do not replace compiled code.".to_string(),
    }
}

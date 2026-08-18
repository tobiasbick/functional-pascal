//! Session mapping onto live-image classification, commit, and rollback.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::Arc;

use fpas_bytecode::VerifiedExecutable;

use super::{DebugSession, stop_at_worker};
use crate::vm::debug::breakpoints as binding;
use crate::vm::debug::live_image::{
    LiveImageClassification, LiveImageReplaceResult, LiveImageUpdateClass, PreparedLiveImageCommit,
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

    /// Return the monotonically increasing live-image version.
    #[must_use]
    pub const fn live_image_version(&self) -> u64 {
        self.live_image_version
    }

    /// Return whether one bounded previous image can be considered for rollback.
    #[must_use]
    pub const fn live_image_rollback_available(&self) -> bool {
        self.previous_executable.is_some()
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

    /// Atomically install a compatible inactive-function update.
    ///
    /// The prior image is retained as the single bounded rollback candidate.
    /// Unchanged images succeed without increasing the version. Rejection or
    /// preparation failure leaves workers, stops, breakpoints, and versions intact.
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
        self.replace_live_image_arc(Arc::new(candidate.clone()))
    }

    /// Roll back to the single retained previous image when it is still compatible.
    ///
    /// A successful rollback is itself a new version and retains the replaced
    /// image as the next bounded rollback candidate.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns [`DebugErrorKind::LiveImageRollbackUnavailable`] when no prior
    /// image is retained, or the normal compatibility/state error otherwise.
    pub fn rollback_live_image(&mut self) -> Result<LiveImageReplaceResult, DebugSessionError> {
        let candidate = self
            .previous_executable
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(rollback_unavailable)?;
        self.replace_live_image_arc(candidate)
    }

    fn replace_live_image_arc(
        &mut self,
        candidate: Arc<VerifiedExecutable>,
    ) -> Result<LiveImageReplaceResult, DebugSessionError> {
        self.require_inspectable("image.replace")?;
        let classification = self.classify_live_image(&candidate);
        if !classification.accepted {
            return Err(incompatible_image(classification.class));
        }
        if classification.class == LiveImageUpdateClass::Unchanged {
            return Ok(LiveImageReplaceResult::new(
                classification,
                false,
                self.live_image_version,
                self.previous_executable.is_some(),
            ));
        }
        if self.recording.capturing() {
            return Err(recording_active());
        }
        let next_version = self
            .live_image_version
            .checked_add(1)
            .ok_or_else(version_exhausted)?;
        let commit = PreparedLiveImageCommit::new(&self.executable, candidate)
            .ok_or_else(commit_preparation_failed)?;
        if !self.runtime.validates_live_image_commit(&commit) {
            return Err(commit_preparation_failed());
        }

        let source_breakpoints = self
            .source_breakpoints
            .iter()
            .map(|breakpoint| {
                binding::bind_source(
                    commit.candidate(),
                    breakpoint.id,
                    breakpoint.requested.clone(),
                )
            })
            .collect();
        let function_breakpoints = self
            .function_breakpoints
            .iter()
            .map(|breakpoint| {
                binding::bind_function(
                    commit.candidate(),
                    breakpoint.id,
                    breakpoint.requested.clone(),
                )
            })
            .collect();
        let previous = Arc::clone(&self.executable);
        self.runtime.commit_live_image(&commit);
        self.executable = Arc::clone(commit.candidate());
        self.previous_executable = Some(previous);
        self.live_image_version = next_version;
        self.source_breakpoints = source_breakpoints;
        self.function_breakpoints = function_breakpoints;
        self.refresh_after_live_image_commit();

        Ok(LiveImageReplaceResult::new(
            classification,
            true,
            self.live_image_version,
            true,
        ))
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
        let candidate = Arc::clone(&self.executable);
        self.replace_live_image_arc(candidate)
    }

    fn refresh_after_live_image_commit(&mut self) {
        let reason = self.last_stop.reason;
        let breakpoint_ids = self.last_stop.breakpoint_ids.clone();
        let diagnostic = self.last_stop.diagnostic.clone();
        let task_id = self.last_stop.task_id;
        if let Some(worker) = self.runtime.worker(task_id) {
            self.last_stop =
                stop_at_worker(&self.executable, worker, reason, breakpoint_ids, diagnostic);
            self.last_stop.task_id = task_id;
        }
        self.invalidate_inspection();
        self.refresh_inspection();
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_workers_share_live_image(&self) -> bool {
        self.runtime.workers_share_live_image(&self.executable)
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_retained_live_image_count(&self) -> usize {
        1 + usize::from(self.previous_executable.is_some())
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

fn rollback_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::LiveImageRollbackUnavailable,
        message: "no previous live image is available for rollback".to_string(),
        hint: "Commit one compatible inactive-function update before requesting rollback."
            .to_string(),
    }
}

fn recording_active() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::InvalidState,
        message: "live-image replacement is unavailable while recording capture is active"
            .to_string(),
        hint: "Start a new debug session before applying a different compiled image. Recording events are never relabeled or used as rollback snapshots."
            .to_string(),
    }
}

fn version_exhausted() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::InvalidState,
        message: "the live-image version counter is exhausted".to_string(),
        hint: "Start a new debug session before applying another compiled image.".to_string(),
    }
}

fn commit_preparation_failed() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::LiveImageIncompatible,
        message: "the live-image update could not remap every retained instruction identity"
            .to_string(),
        hint: "Keep the current image and rebuild without changing active function bodies or function identities."
            .to_string(),
    }
}

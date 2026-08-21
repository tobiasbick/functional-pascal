//! Live-image classification and reject-before-commit.

use super::record::{DebugRecord, ResponseBody};
use super::reply::{invalid_state, ok, session_error};
use super::{DebugEngine, DebugStatus};

impl DebugEngine {
    pub(super) fn classify_live_image(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let candidate = match self.reloader.as_mut() {
            Some(reloader) => match reloader() {
                Ok(reloaded) => Some(reloaded.into_parts().0),
                Err(error) => return vec![session_error(request_id, command, error)],
            },
            None => None,
        };
        let Some(session) = self.actor.session() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        let classification = candidate.as_ref().map_or_else(
            || session.classify_current_live_image(),
            |candidate| session.classify_live_image(candidate),
        );
        vec![ok(
            request_id,
            command,
            live_image_body(
                classification.class,
                classification.accepted,
                false,
                session.live_image_version(),
                session.live_image_rollback_available(),
            ),
        )]
    }

    pub(super) fn reload_live_image(&mut self, request_id: u64, command: &str) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let reloaded = match self.reloader.as_mut() {
            Some(reloader) => match reloader() {
                Ok(reloaded) => Some(reloaded.into_parts()),
                Err(error) => return vec![session_error(request_id, command, error)],
            },
            None => None,
        };
        let result = {
            let Some(session) = self.actor.session_mut() else {
                return vec![invalid_state(request_id, command, self.status)];
            };
            match reloaded.as_ref() {
                Some((candidate, _)) => session.replace_live_image(candidate),
                None => session.replace_current_live_image(),
            }
        };
        match result {
            Ok(result) => {
                if result.applied
                    && let Some((_, sources)) = reloaded
                {
                    self.commit_sources(sources);
                }
                vec![ok(request_id, command, replacement_body(result))]
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn rollback_live_image(
        &mut self,
        request_id: u64,
        command: &str,
    ) -> Vec<DebugRecord> {
        if !matches!(self.status, DebugStatus::Initialized | DebugStatus::Stopped) {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.rollback_live_image() {
            Ok(result) => {
                if result.applied {
                    self.rollback_sources();
                }
                vec![ok(request_id, command, replacement_body(result))]
            }
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    /// Reject an incompatible candidate before the live image can change.
    ///
    /// Protocol `reload` / `image.replace` gate the current executable.
    /// Tests and later CLI reload pass a second compiled image here.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns a session error when the actor is not ready or the candidate is
    /// incompatible.
    pub fn replace_live_image(
        &mut self,
        candidate: &fpas_bytecode::VerifiedExecutable,
    ) -> Result<fpas_vm::LiveImageReplaceResult, fpas_vm::DebugSessionError> {
        let Some(session) = self.actor.session_mut() else {
            return Err(fpas_vm::DebugSessionError {
                kind: fpas_vm::DebugErrorKind::InvalidState,
                message: "live-image replace is invalid while the debug session is not inspectable"
                    .to_string(),
                hint: "Wait for initialize or a stopped event before replacing the live image."
                    .to_string(),
            });
        };
        session.replace_live_image(candidate)
    }

    fn commit_sources(&mut self, sources: Vec<crate::DebugSourceContent>) {
        let previous = std::mem::replace(&mut self.sources, sources);
        self.previous_sources = Some(previous);
        self.source_revision = self.source_revision.saturating_add(1);
    }

    fn rollback_sources(&mut self) {
        let Some(previous) = self.previous_sources.take() else {
            return;
        };
        let replaced = std::mem::replace(&mut self.sources, previous);
        self.previous_sources = Some(replaced);
        self.source_revision = self.source_revision.saturating_add(1);
    }
}

fn replacement_body(result: fpas_vm::LiveImageReplaceResult) -> ResponseBody {
    live_image_body(
        result.class,
        result.accepted,
        result.applied,
        result.version,
        result.rollback_available,
    )
}

fn live_image_body(
    class: fpas_vm::LiveImageUpdateClass,
    accepted: bool,
    applied: bool,
    version: u64,
    rollback_available: bool,
) -> ResponseBody {
    ResponseBody::LiveImage {
        class,
        accepted,
        applied,
        version,
        rollback_available,
    }
}

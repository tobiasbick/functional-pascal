//! Compare full inventories before resolving a selector that a successful edit invalidates.

use super::*;
use crate::{ANALYSIS_BUILD_ID, CancellationToken, SourceQueryError, SourceSession, source_digest};

impl SourceSession {
    /// Revalidates a saved plan or editor handle and regenerates exact edits from its recipe.
    /// This performs no source publication and grants no authority to the supplied edit paths.
    pub fn revalidate_refactor(
        &mut self,
        plan: &RefactorPlan,
        cancellation: &CancellationToken,
    ) -> Result<RefactorPreparation, SourceQueryError> {
        plan.validate()?;
        cancellation.check()?;
        if plan.analysis_identity != ANALYSIS_BUILD_ID || &plan.context != self.context() {
            return Err(stale(
                "The semantic implementation or selected context changed.",
            ));
        }
        for edit in &plan.edits {
            self.editable_source(std::path::Path::new(&edit.path))?;
        }
        let inventory = &self.snapshot().inventory;
        if inventory == &plan.after && inventory != &plan.before {
            // Prove that exact reverse edits reconstruct the claimed original bytes. Even an
            // already-applied report must reject corrupt original text and impossible offsets.
            self.validate_applied_bytes(plan)?;
            if !self.verify()? {
                return Err(stale("Inputs changed during revalidation."));
            }
            return Ok(RefactorPreparation {
                status: RefactorStatus::AlreadyApplied,
                plan: Some(plan.clone()),
                changes: None,
                blockers: Vec::new(),
                limitations: super::prepare::limitations(),
                diagnostics: Vec::new(),
            });
        }
        if inventory != &plan.before {
            return Err(stale(
                "The complete current inventory matches neither the before nor the expected after state.",
            ));
        }
        let prepared = self.prepare_refactor(&plan.recipe, cancellation)?;
        if prepared.plan.as_ref() != Some(plan) {
            return Err(stale(
                "The checked recipe no longer regenerates this exact plan.",
            ));
        }
        Ok(prepared)
    }

    fn validate_applied_bytes(&self, plan: &RefactorPlan) -> Result<(), SourceQueryError> {
        for file in self.inventory().files() {
            let path = crate::identity_path(&file.path).map_err(|e| stale(e.to_string()))?;
            let edits = plan
                .edits
                .iter()
                .filter(|e| e.path == path)
                .collect::<Vec<_>>();
            if edits.is_empty() {
                continue;
            }
            let source = std::str::from_utf8(&file.bytes).map_err(|e| stale(e.to_string()))?;
            let mut reversed = String::new();
            let mut after_end = 0usize;
            let mut before_end = 0usize;
            for edit in edits {
                let start = after_end
                    .checked_add(edit.start - before_end)
                    .ok_or_else(|| stale("Invalid edit offsets."))?;
                let end = start
                    .checked_add(edit.new_text.len())
                    .ok_or_else(|| stale("Invalid edit offsets."))?;
                let gap = source
                    .get(after_end..start)
                    .ok_or_else(|| stale("Invalid edit boundary."))?;
                if source.get(start..end) != Some(edit.new_text.as_str()) {
                    return Err(super::plan::invalid(
                        "Applied bytes do not match the exact planned replacements.",
                    ));
                }
                reversed.push_str(gap);
                reversed.push_str(&edit.old_text);
                after_end = end;
                before_end = edit.start + edit.length;
            }
            reversed.push_str(
                source
                    .get(after_end..)
                    .ok_or_else(|| stale("Invalid edit boundary."))?,
            );
            if plan
                .before
                .files
                .iter()
                .find(|f| f.path == path)
                .is_none_or(|f| {
                    f.bytes != reversed.len() || f.sha256 != source_digest(reversed.as_bytes())
                })
            {
                return Err(super::plan::invalid(
                    "Original bytes do not match the recorded before inventory.",
                ));
            }
        }
        Ok(())
    }
}

fn stale(message: impl Into<String>) -> SourceQueryError {
    SourceQueryError::new("stale_plan", message)
}

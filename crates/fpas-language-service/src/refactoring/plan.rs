//! Portable exact plans are validation data, never filesystem write authority.

mod digest;
mod validation;

use super::{RefactorRecipe, recipe::valid_path};
use crate::{
    ANALYSIS_BUILD_ID, CheckedChanges, InventoryIdentity, ScopeContext, SourceQueryError,
    identity_path, source_digest,
};
use serde::{Deserialize, Serialize};

/// Maximum public plan/report bytes; edits are rejected rather than clipped.
pub const REFACTOR_JSON_BYTES: usize = 8 * 1024 * 1024;

/// One exact replacement in the original UTF-8 coordinate space.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedEdit {
    /// Canonical absolute source identity.
    pub path: String,
    /// Original inclusive byte start.
    pub start: usize,
    /// Original byte length.
    pub length: usize,
    /// Exact original bytes, represented as UTF-8.
    pub old_text: String,
    /// Exact replacement bytes, represented as UTF-8.
    pub new_text: String,
}

/// Deterministic recipe and complete before/after inventories for a checked transformation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefactorPlan {
    /// Encoding version, currently one.
    pub plan_version: u32,
    /// Shared semantic implementation identity.
    pub analysis_identity: String,
    /// Authoritative context to reconstruct, independent of the caller's working directory.
    pub context: ScopeContext,
    /// Exact normalized transformation to regenerate against the before inventory.
    pub recipe: RefactorRecipe,
    /// Complete original read set.
    pub before: InventoryIdentity,
    /// Expected read set after exactly these edits.
    pub after: InventoryIdentity,
    /// Stable path/start ordered, nonoverlapping edits.
    pub edits: Vec<PlannedEdit>,
    /// Canonical content checksum, not a signature or permission grant.
    pub digest: String,
}

impl RefactorPlan {
    pub(super) fn new(
        context: ScopeContext,
        recipe: RefactorRecipe,
        before: InventoryIdentity,
        changes: &CheckedChanges,
    ) -> Result<Self, SourceQueryError> {
        let mut after = before.clone();
        for change in &changes.changes {
            let path = identity_path(&change.path).map_err(|e| invalid(e.to_string()))?;
            let input = after
                .files
                .iter_mut()
                .find(|f| f.path == path)
                .ok_or_else(|| invalid("An edit is outside the frozen inventory."))?;
            input.sha256 = source_digest(change.replacement.as_bytes());
            input.bytes = change.replacement.len();
        }
        let edits = changes
            .edits
            .iter()
            .map(|edit| {
                Ok(PlannedEdit {
                    path: identity_path(&edit.path).map_err(|e| invalid(e.to_string()))?,
                    start: edit.range.offset(),
                    length: edit.range.length(),
                    old_text: edit
                        .snapshot
                        .source()
                        .get(edit.range.offset()..edit.range.end())
                        .ok_or_else(|| invalid("Invalid original edit range."))?
                        .into(),
                    new_text: edit.new_text.clone(),
                })
            })
            .collect::<Result<Vec<_>, SourceQueryError>>()?;
        let mut plan = Self {
            plan_version: 1,
            analysis_identity: ANALYSIS_BUILD_ID.into(),
            context,
            recipe,
            before,
            after,
            edits,
            digest: String::new(),
        };
        plan.digest = plan.content_digest();
        plan.validate()?;
        Ok(plan)
    }
}

pub(super) fn invalid(message: impl Into<String>) -> SourceQueryError {
    SourceQueryError::new("invalid_plan", message)
}

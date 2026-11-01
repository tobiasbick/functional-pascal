//! Reject malformed plans before loading their recorded context or allocating replacements.

use super::super::recipe::valid_digest;
use super::*;
use crate::{QueryPolicy, ScopeKind};

impl RefactorPlan {
    /// Validates untrusted structure and checksum; application must still reconstruct and check.
    pub fn validate(&self) -> Result<(), SourceQueryError> {
        let context = &self.context;
        if self.plan_version != 1
            || !valid_digest(&self.analysis_identity)
            || !valid_digest(&self.digest)
            || context.policy != QueryPolicy::CheckedContext
            || context.kind == ScopeKind::Folder
            || !valid_path(&context.input)
            || !valid_path(&context.root)
            || context
                .standard_library
                .as_deref()
                .is_some_and(|p| !valid_path(p))
            || context.projects.iter().any(|p| !valid_path(p))
            || !sorted(&context.projects)
        {
            return Err(invalid("Malformed or unsupported plan context/version."));
        }
        self.recipe.validate().map_err(|e| invalid(e.message))?;
        match &self.recipe {
            RefactorRecipe::AddParameter {
                name,
                r#type,
                mode,
                argument,
                ..
            } => {
                super::super::add_parameter::fragments::Parameter {
                    name,
                    type_text: r#type,
                    mode: (*mode).into(),
                    argument,
                }
                .validate()
                .map_err(|e| invalid(e.message))?;
            }
            RefactorRecipe::Rename { name, .. } => {
                crate::navigation::validate_identifier(name).map_err(|e| invalid(e.to_string()))?
            }
            RefactorRecipe::ExtractVariable { name, .. } => {
                crate::navigation::validate_identifier(name.as_deref().ok_or_else(|| {
                    invalid("A prepared extraction must record its generated name.")
                })?)
                .map_err(|e| invalid(e.to_string()))?
            }
            RefactorRecipe::OrganizeImports { .. } => {}
        }
        inventory(&self.before)?;
        inventory(&self.after)?;
        if self.before.aliases != self.after.aliases
            || self.before.selected != self.after.selected
            || self.before.globs != self.after.globs
            || self.before.files.len() != self.after.files.len()
        {
            return Err(invalid(
                "A text refactoring cannot change input membership or aliases.",
            ));
        }
        let mut previous: Option<&PlannedEdit> = None;
        let mut bytes = 0usize;
        for edit in &self.edits {
            bytes = bytes
                .saturating_add(edit.old_text.len())
                .saturating_add(edit.new_text.len());
            if !valid_path(&edit.path)
                || edit.length != edit.old_text.len()
                || edit.old_text == edit.new_text
                || edit.start.checked_add(edit.length).is_none()
                || previous.is_some_and(|p| {
                    p.path > edit.path
                        || p.path == edit.path
                            && (p.start >= edit.start || p.start + p.length > edit.start)
                })
                || !self
                    .before
                    .files
                    .iter()
                    .any(|f| f.path == edit.path && edit.start + edit.length <= f.bytes)
            {
                return Err(invalid("Malformed, unordered or overlapping source edits."));
            }
            if !crate::workspace::path_containment::contains(
                std::path::Path::new(&context.root),
                std::path::Path::new(&edit.path),
            ) || context.standard_library.as_ref().is_some_and(|root| {
                crate::workspace::path_containment::contains(
                    std::path::Path::new(root),
                    std::path::Path::new(&edit.path),
                )
            }) {
                return Err(invalid(
                    "An edit is outside the context's writable source root.",
                ));
            }
            previous = Some(edit);
        }
        if bytes > REFACTOR_JSON_BYTES {
            return Err(SourceQueryError::new(
                "plan_limit",
                "Exact edits exceed the public plan limit.",
            ));
        }
        for (before, after) in self.before.files.iter().zip(&self.after.files) {
            let edits = self
                .edits
                .iter()
                .filter(|e| e.path == before.path)
                .collect::<Vec<_>>();
            let expected = edits.iter().try_fold(before.bytes, |n, e| {
                n.checked_sub(e.old_text.len())?
                    .checked_add(e.new_text.len())
            });
            if before.path != after.path
                || expected != Some(after.bytes)
                || edits.is_empty() && before != after
            {
                return Err(invalid(
                    "The expected inventory does not match the exact edits.",
                ));
            }
        }
        if self.content_digest() != self.digest {
            return Err(invalid("The plan content checksum does not match."));
        }
        Ok(())
    }
}

fn inventory(inventory: &InventoryIdentity) -> Result<(), SourceQueryError> {
    use fpas_project::source_inputs::{MAX_FILE_BYTES, MAX_FILES, MAX_TOTAL_BYTES};
    if inventory.files.len() > MAX_FILES
        || inventory
            .files
            .iter()
            .any(|f| !valid_path(&f.path) || !valid_digest(&f.sha256) || f.bytes > MAX_FILE_BYTES)
        || inventory.files.windows(2).any(|w| w[0].path >= w[1].path)
        || inventory
            .files
            .iter()
            .fold(0usize, |n, f| n.saturating_add(f.bytes))
            > MAX_TOTAL_BYTES
        || inventory
            .aliases
            .iter()
            .any(|(a, b)| !valid_path(a) || !valid_path(b))
        || !sorted(&inventory.aliases)
        || !sorted(&inventory.selected)
        || inventory.selected.iter().any(|p| !valid_path(p))
        || !sorted(&inventory.globs)
        || inventory.globs.iter().any(|g| {
            !valid_path(&g.root) || !sorted(&g.members) || g.members.iter().any(|p| !valid_path(p))
        })
    {
        return Err(invalid("Malformed or oversized plan inventory."));
    }
    Ok(())
}

fn sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|p| p[0] < p[1])
}

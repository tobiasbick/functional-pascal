//! One checked recipe owner for CLI previews, LSP edits and apply-time regeneration.

use super::*;
use crate::{
    CancellationToken, QueryPolicy, RenameError, SourceQueryError, SourceQueryOperation,
    SourceQueryTarget, SourceSelector, SourceSession, identity_path,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

impl SourceSession {
    /// Prepares exact edits against the complete current scope, without changing any source.
    pub fn prepare_refactor(
        &mut self,
        recipe: &RefactorRecipe,
        cancellation: &CancellationToken,
    ) -> Result<RefactorPreparation, SourceQueryError> {
        recipe.validate()?;
        if self.context().policy != QueryPolicy::CheckedContext {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Refactoring requires a checked context.",
            ));
        }
        let mut blockers = Vec::new();
        let prepared = self.prepare_recipe(recipe, &mut blockers, cancellation);
        cancellation.check()?;
        if !self.verify()? {
            return Ok(blocked(SourceQueryError::new(
                "source_changed",
                "Inputs changed while preparing the refactoring.",
            )));
        }
        let (recipe, changes) = match prepared {
            Ok(prepared) => prepared,
            Err(error) if error.code == "invalid_request" => return Err(error),
            Err(error) => {
                let mut result = blocked(error);
                if !blockers.is_empty() {
                    result.blockers = blockers;
                }
                return Ok(result);
            }
        };
        let no_change = changes.edits.is_empty();
        let adds_parameter = matches!(recipe, RefactorRecipe::AddParameter { .. });
        let plan = RefactorPlan::new(
            self.context().clone(),
            recipe,
            self.snapshot().inventory.clone(),
            &changes,
        )?;
        Ok(RefactorPreparation {
            status: if no_change {
                RefactorStatus::NoChange
            } else {
                RefactorStatus::Prepared
            },
            plan: Some(plan),
            changes: Some(changes),
            blockers: if no_change {
                vec![RefactorBlocker {
                    code: "no_change".into(),
                    message: "The eligible transformation has nothing to change.".into(),
                    path: None,
                    start: None,
                    length: None,
                }]
            } else {
                Vec::new()
            },
            limitations: {
                let mut values = limitations();
                if adds_parameter {
                    values.push("The new argument runs after existing written arguments, once when reached; its effects, failure or try propagation may prevent the call.".into());
                }
                values
            },
            diagnostics: Vec::new(),
        })
    }

    fn prepare_recipe(
        &mut self,
        recipe: &RefactorRecipe,
        blockers: &mut Vec<RefactorBlocker>,
        cancellation: &CancellationToken,
    ) -> Result<(RefactorRecipe, CheckedChanges), SourceQueryError> {
        match recipe {
            RefactorRecipe::AddParameter { .. } => {
                add_parameter::prepare(self, recipe, blockers, cancellation)
            }
            RefactorRecipe::Rename { selector, name } => {
                let target = self.refactor_target(selector, cancellation)?;
                let reference = target.target_ref.ok_or_else(|| {
                    SourceQueryError::new(
                        "unsupported_target",
                        "The selected declaration has no verified editable source identity.",
                    )
                })?;
                let path = Path::new(&reference.file);
                self.editable_source(path)?;
                let changes = self
                    .service()
                    .checked_rename(path, reference.start, name, cancellation)
                    .map_err(rename_error)?
                    .value;
                Ok((
                    RefactorRecipe::Rename {
                        selector: SourceSelector::Reference {
                            target_ref: Box::new(reference),
                        },
                        name: name.clone(),
                    },
                    changes,
                ))
            }
            RefactorRecipe::ExtractVariable {
                path,
                start,
                end,
                name,
            } => {
                let path = self.editable_source(Path::new(path))?;
                // Establish complete before facts even when the selection is ineligible.
                self.service()
                    .validate_source_edits(&path, &[], cancellation)?;
                let context = self.service().navigation_context(&path)?;
                let index = context
                    .target_index
                    .ok_or_else(|| unsupported("source_unavailable"))?;
                let edits = extract::extract_variable(&context.documents, index, *start..*end)
                    .ok_or_else(|| unsupported("unsafe_extraction"))?;
                let generated = edits
                    .last()
                    .map(|e| e.new_text.clone())
                    .ok_or_else(|| unsupported("unsafe_extraction"))?;
                if name.as_ref().is_some_and(|name| name != &generated) {
                    return Err(SourceQueryError::new(
                        "stale_plan",
                        "The deterministic extracted name changed.",
                    ));
                }
                let changes = self.check_local(&path, &context.snapshot, edits, cancellation)?;
                Ok((
                    RefactorRecipe::ExtractVariable {
                        path: identity_path(&path).map_err(|e| {
                            SourceQueryError::new("unsupported_path_identity", e.to_string())
                        })?,
                        start: *start,
                        end: *end,
                        name: Some(generated),
                    },
                    changes,
                ))
            }
            RefactorRecipe::OrganizeImports { path } => {
                let path = self.editable_source(Path::new(path))?;
                self.service()
                    .validate_source_edits(&path, &[], cancellation)?;
                let context = self.service().navigation_context(&path)?;
                let index = context
                    .target_index
                    .ok_or_else(|| unsupported("source_unavailable"))?;
                let edits = imports::organize(&context.documents, index).map_err(unsupported)?;
                let changes = self.check_local(&path, &context.snapshot, edits, cancellation)?;
                Ok((
                    RefactorRecipe::OrganizeImports {
                        path: identity_path(&path).map_err(|e| {
                            SourceQueryError::new("unsupported_path_identity", e.to_string())
                        })?,
                    },
                    changes,
                ))
            }
        }
    }

    pub(crate) fn refactor_target(
        &mut self,
        selector: &SourceSelector,
        cancellation: &CancellationToken,
    ) -> Result<SourceQueryTarget, SourceQueryError> {
        let result = match selector {
            SourceSelector::Position { path, offset } => self.query_position(
                Path::new(path),
                *offset,
                SourceQueryOperation::Definition,
                1000,
                cancellation,
            )?,
            SourceSelector::Symbol { name } => {
                self.query_symbol(name, SourceQueryOperation::Definition, 1000, cancellation)?
            }
            SourceSelector::Reference { target_ref } => self.query_reference(
                target_ref,
                SourceQueryOperation::Definition,
                1000,
                cancellation,
            )?,
        };
        if !result.reasons.is_empty() {
            return Err(SourceQueryError {
                code: "analysis_incomplete",
                message: format!(
                    "Declaration selection is incomplete: {}.",
                    result.reasons.into_iter().collect::<Vec<_>>().join(", ")
                ),
                diagnostics: result.diagnostics,
            });
        }
        if let Some(reason) = result.selection_error {
            return Err(unsupported(reason));
        }
        result.target.ok_or_else(|| unsupported("no_target"))
    }

    pub(crate) fn editable_source(&mut self, path: &Path) -> Result<PathBuf, SourceQueryError> {
        let path = std::fs::canonicalize(path)
            .map_err(|e| SourceQueryError::new("source_read", e.to_string()))?;
        let identity = identity_path(&path)
            .map_err(|e| SourceQueryError::new("unsupported_path_identity", e.to_string()))?;
        if !self
            .snapshot()
            .inventory
            .files
            .iter()
            .any(|f| f.path == identity)
            || !crate::workspace::path_containment::contains(Path::new(&self.context().root), &path)
            || self.service().is_editor_api_source(&path)
            || self
                .context()
                .standard_library
                .as_ref()
                .is_some_and(|root| {
                    crate::workspace::path_containment::contains(Path::new(root), &path)
                })
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("fpas"))
        {
            return Err(unsupported("outside_edit_scope"));
        }
        Ok(path)
    }

    fn check_local(
        &mut self,
        path: &Path,
        snapshot: &Arc<crate::DocumentSnapshot>,
        edits: Vec<crate::SemanticEdit>,
        cancellation: &CancellationToken,
    ) -> Result<CheckedChanges, SourceQueryError> {
        let edits = edits
            .into_iter()
            .map(|edit| SourceEdit {
                path: path.into(),
                snapshot: Arc::clone(snapshot),
                range: edit.span,
                new_text: edit.new_text,
            })
            .collect::<Vec<_>>();
        self.service()
            .validate_source_edits(path, &edits, cancellation)
    }
}

fn rename_error(error: RenameError) -> SourceQueryError {
    match error {
        RenameError::Service(error) => error.into(),
        RenameError::Validation(error) => error,
        error => SourceQueryError::new("unsupported_refactoring", error.to_string()),
    }
}

fn unsupported(reason: &'static str) -> SourceQueryError {
    SourceQueryError::new(
        reason,
        match reason {
            "import_comments" => {
                "Comments inside the uses clause prevent safe import organization."
            }
            "unsafe_extraction" => {
                "The selection cannot be extracted without changing evaluation order, frequency or binding."
            }
            "outside_edit_scope" => {
                "Affected sources must be inside the checked context and outside generated APIs or the standard library."
            }
            "no_target" => "No declaration resolves at the selected position.",
            "ambiguous_target" => "Several declarations match the selected name.",
            _ => "The selected source form is not supported for this refactoring.",
        },
    )
}

fn blocked(error: SourceQueryError) -> RefactorPreparation {
    let incomplete = matches!(
        error.code,
        "analysis_incomplete"
            | "analysis_unavailable"
            | "source_read"
            | "source_changed"
            | "source_limit"
            | "query_cancelled"
            | "ambiguous_context"
    );
    RefactorPreparation {
        status: if incomplete {
            RefactorStatus::Incomplete
        } else {
            RefactorStatus::Unsupported
        },
        plan: None,
        changes: None,
        blockers: vec![RefactorBlocker {
            code: error.code.into(),
            message: error.message,
            path: None,
            start: None,
            length: None,
        }],
        limitations: limitations(),
        diagnostics: error.diagnostics,
    }
}

pub(super) fn limitations() -> Vec<String> {
    vec!["Only consumers in the selected checked context are validated.".into()]
}

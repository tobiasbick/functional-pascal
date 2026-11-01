//! Adapter-independent recipes and explicit preparation outcomes.

use crate::{CheckedChanges, SourceQueryError, TargetReference, identity_path};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// An exact declaration selector; wire adapters convert their own position encodings first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceSelector {
    /// A UTF-8 byte position in an absolute source path.
    Position {
        /// Absolute identity path.
        path: String,
        /// UTF-8 byte offset.
        offset: usize,
    },
    /// An exact short or owner-qualified declaration name.
    Symbol {
        /// Case-insensitive declaration spelling.
        name: String,
    },
    /// A checked declaration in a complete immutable scope.
    Reference {
        /// Portable declaration reference, never retargeted heuristically.
        target_ref: Box<TargetReference>,
    },
}

impl SourceSelector {
    pub(crate) fn validate(&self) -> Result<(), SourceQueryError> {
        match self {
            Self::Position { path, .. } if valid_path(path) => Ok(()),
            Self::Symbol { name } if !name.trim().is_empty() => Ok(()),
            Self::Reference { target_ref } => target_ref.validate(),
            _ => Err(SourceQueryError::new(
                "invalid_request",
                "Invalid declaration selector.",
            )),
        }
    }
}

/// A reproducible transformation, independent of CLI artifacts and editor application.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RefactorRecipe {
    /// Append one checked parameter and an explicit argument to every supported direct call.
    AddParameter {
        /// Declaration whose signature changes.
        selector: SourceSelector,
        /// New parameter identifier.
        name: String,
        /// Complete type expression in the declaration's scope.
        r#type: String,
        /// Value or writable-reference passing.
        mode: ParameterMode,
        /// Complete expression interpreted separately in every caller's scope.
        argument: String,
    },
    /// Rename one checked declaration and all proven uses.
    Rename {
        /// Declaration to rename.
        selector: SourceSelector,
        /// New identifier spelling.
        name: String,
    },
    /// Extract an exact expression without changing evaluation order or frequency.
    ExtractVariable {
        /// Absolute identity path.
        path: String,
        /// Inclusive UTF-8 byte start.
        start: usize,
        /// Exclusive UTF-8 byte end.
        end: usize,
        /// Deterministic generated name; absent in a request, present in a prepared recipe.
        name: Option<String>,
    },
    /// Organize one source's imports while preserving initialization and comments.
    OrganizeImports {
        /// Absolute identity path.
        path: String,
    },
}

impl RefactorRecipe {
    /// Validates structure before any recorded filesystem path is loaded.
    pub fn validate(&self) -> Result<(), SourceQueryError> {
        let valid = match self {
            Self::AddParameter {
                selector,
                name,
                r#type,
                argument,
                ..
            } => {
                selector.validate()?;
                [name, r#type, argument]
                    .iter()
                    .all(|s| !s.trim().is_empty())
            }
            Self::Rename { selector, name } => {
                selector.validate()?;
                !name.trim().is_empty()
            }
            Self::ExtractVariable {
                path,
                start,
                end,
                name,
            } => valid_path(path) && start < end && name.as_ref().is_none_or(|s| !s.is_empty()),
            Self::OrganizeImports { path } => valid_path(path),
        };
        if valid {
            Ok(())
        } else {
            Err(SourceQueryError::new(
                "invalid_request",
                "Malformed refactoring recipe.",
            ))
        }
    }
}

/// Existing FPAS parameter passing modes, independent of parser representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterMode {
    /// Evaluate the value once in the caller.
    Value,
    /// Borrow writable caller storage using the existing `var` rules.
    Var,
}

impl From<ParameterMode> for fpas_parser::ParamMode {
    fn from(mode: ParameterMode) -> Self {
        match mode {
            ParameterMode::Value => Self::Value,
            ParameterMode::Var => Self::Var,
        }
    }
}

/// Semantic outcome; lack of edits alone never determines the status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefactorStatus {
    /// Checked edits are ready for review and revalidation.
    Prepared,
    /// The eligible transformation has nothing to change.
    NoChange,
    /// Complete facts show that this transformation cannot be performed safely.
    Unsupported,
    /// Missing or invalid analysis prevents a safety decision.
    Incomplete,
    /// The complete saved inventory already equals the planned result.
    AlreadyApplied,
    /// An adapter successfully published the revalidated exact edits.
    Applied,
}

/// A stable reason why preparation cannot yield an applicable plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefactorBlocker {
    /// Machine-readable semantic reason.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
    /// Exact source path, if attributable to a site.
    pub path: Option<String>,
    /// Exact UTF-8 byte start, if known.
    pub start: Option<usize>,
    /// Exact UTF-8 byte length, if known.
    pub length: Option<usize>,
}

/// Shared semantic preparation with exact edits; this value never publishes source.
#[derive(Debug)]
pub struct RefactorPreparation {
    /// Explicit semantic outcome.
    pub status: RefactorStatus,
    /// Present only for checked prepared/no-change results.
    pub plan: Option<super::RefactorPlan>,
    /// Exact replacement buffers for the adapter, never trusted from a supplied plan.
    pub changes: Option<CheckedChanges>,
    /// Semantic blockers or the explicit no-change reason.
    pub blockers: Vec<RefactorBlocker>,
    /// Scope and effect limitations, independent from edit eligibility.
    pub limitations: Vec<String>,
    /// Original attributed compiler diagnostics.
    pub diagnostics: Vec<fpas_diagnostics::FileDiagnostic>,
}

pub(crate) fn valid_path(path: &str) -> bool {
    Path::new(path).is_absolute() && identity_path(Path::new(path)).is_ok_and(|p| p == path)
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

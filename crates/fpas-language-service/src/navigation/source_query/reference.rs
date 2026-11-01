//! Checked references never fall back to a similar declaration after inputs change.

use super::{ScopeContext, SourceQueryOperation, SourceQueryResult, SourceSession};
use crate::{CancellationToken, LanguageServiceError, SymbolKind, identity_path, source_digest};
use fpas_diagnostics::FileDiagnostic;
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

/// A declaration in an exact complete scope. This is validation data, not write authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetReference {
    /// Reference encoding version, currently one.
    pub version: u32,
    /// Information needed to reconstruct the original context.
    pub context: ScopeContext,
    /// Content-derived complete-scope identity.
    pub snapshot_id: String,
    /// Shared semantic build identity.
    pub analysis_identity: String,
    /// Owning project manifest, absent for a loose source or intrinsic declaration.
    pub owner: Option<String>,
    /// Declaration category.
    pub kind: SymbolKind,
    /// Original owner-qualified spelling, used for verification rather than retargeting.
    pub qualified_name: String,
    /// Canonical physical defining source path.
    pub file: String,
    /// Start of the defining identifier in UTF-8 bytes.
    pub start: usize,
    /// Length of the defining identifier in UTF-8 bytes.
    pub length: usize,
    /// Exact defining source digest.
    pub file_sha256: String,
}

/// Stable source-tool failure with original compiler attribution retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQueryError {
    /// Stable machine-readable failure class.
    pub code: &'static str,
    /// Human-readable explanation, not an input for classification.
    pub message: String,
    /// Original source-attributed FP diagnostics.
    pub diagnostics: Vec<FileDiagnostic>,
}

impl SourceQueryError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            diagnostics: Vec::new(),
        }
    }
}
impl fmt::Display for SourceQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for SourceQueryError {}
impl From<LanguageServiceError> for SourceQueryError {
    fn from(error: LanguageServiceError) -> Self {
        Self {
            code: if error == LanguageServiceError::Cancelled {
                "query_cancelled"
            } else {
                "analysis_unavailable"
            },
            message: error.to_string(),
            diagnostics: error.diagnostics(),
        }
    }
}

impl SourceSession {
    /// Performs a positional query and issues a reference only after final scope verification.
    pub fn query_position(
        &mut self,
        path: &Path,
        offset: usize,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceQueryResult, SourceQueryError> {
        if self.context().policy != super::QueryPolicy::CheckedContext
            || !(1..=10000).contains(&limit)
        {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Declaration selection requires checked_context policy and a limit between 1 and 10000.",
            ));
        }
        let path = std::fs::canonicalize(path)
            .and_then(|p| identity_path(&p))
            .map_err(|e| SourceQueryError::new("source_read", e.to_string()))?;
        if !self
            .snapshot()
            .inventory
            .files
            .iter()
            .any(|f| f.path == path)
        {
            return Err(SourceQueryError::new(
                "outside_context",
                "The selected source is outside the checked context.",
            ));
        }
        if self
            .service()
            .workspace()
            .projects()
            .iter()
            .filter(|p| p.owns_source(Path::new(&path)))
            .count()
            > 1
        {
            return Err(SourceQueryError::new(
                "ambiguous_context",
                "Several selected projects directly own this source; choose one project context.",
            ));
        }
        let mut result = self.service().source_query(
            Path::new(&path),
            offset,
            operation,
            limit,
            cancellation,
        )?;
        if matches!(
            operation,
            SourceQueryOperation::Callers | SourceQueryOperation::Callees
        ) && let Some(target) = &result.target
        {
            let (calls, diagnostics) =
                self.call_relations(target, operation, limit, cancellation)?;
            result.reasons.extend(calls.reasons.iter().copied());
            result.diagnostics.extend(diagnostics);
            result.calls = Some(calls);
        }
        if !self.verify()? {
            result.reasons.insert("source_changed");
        }
        let verified = result.reasons.iter().all(|r| {
            matches!(
                *r,
                "result_limit" | "dynamic_targets" | "body_unavailable" | "unsupported_target"
            )
        });
        if let Some(target) = &mut result.target {
            if !verified {
                target.reference_unavailable = Some("analysis_incomplete");
            } else if let Some((snapshot, symbol)) =
                target.snapshot.as_ref().zip(target.symbol.as_ref())
            {
                let file = std::fs::canonicalize(snapshot.path())
                    .and_then(|p| identity_path(&p))
                    .map_err(|e| {
                        SourceQueryError::new("unsupported_path_identity", e.to_string())
                    })?;
                let mut owners = self
                    .service()
                    .workspace()
                    .projects()
                    .iter()
                    .filter(|p| p.contains_source(snapshot.path()))
                    .map(|p| match p.source_origin(snapshot.path()) {
                        fpas_project::SourceOrigin::Own => identity_path(p.manifest_path()),
                        fpas_project::SourceOrigin::Library(path) => identity_path(&path),
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| {
                        SourceQueryError::new("unsupported_path_identity", e.to_string())
                    })?;
                if owners.is_empty()
                    && self
                        .service()
                        .standard_library_source_paths()
                        .contains(&snapshot.path().to_path_buf())
                    && let Some(root) = &self.context().standard_library
                {
                    owners.push(format!("{root}/stdlib.fpasprj"));
                }
                owners.sort();
                owners.dedup();
                if owners.len() > 1 {
                    target.reference_unavailable = Some("ambiguous_context");
                } else {
                    target.target_ref = Some(TargetReference {
                        version: 1,
                        context: self.context().clone(),
                        snapshot_id: self.snapshot().id.clone(),
                        analysis_identity: self.snapshot().analysis_identity.clone(),
                        owner: owners.into_iter().next(),
                        kind: symbol.kind,
                        qualified_name: symbol.qualified_name.clone(),
                        file,
                        start: symbol.selection_span.offset(),
                        length: symbol.selection_span.length(),
                        file_sha256: source_digest(snapshot.source().as_bytes()),
                    });
                    target.reference_unavailable = None;
                }
            }
        }
        Ok(result)
    }

    /// Reuses a checked declaration only when its entire original semantic identity still matches.
    pub fn query_reference(
        &mut self,
        reference: &TargetReference,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceQueryResult, SourceQueryError> {
        reference.validate()?;
        if &reference.context != self.context()
            || reference.snapshot_id != self.snapshot().id
            || reference.analysis_identity != self.snapshot().analysis_identity
        {
            return Err(SourceQueryError::new(
                "stale_target",
                "The target's context or source snapshot has changed.",
            ));
        }
        if !self.snapshot().inventory.files.iter().any(|file| {
            file.path == reference.file
                && file.sha256 == reference.file_sha256
                && reference.start + reference.length <= file.bytes
        }) {
            return Err(SourceQueryError::new(
                "stale_target",
                "The defining source no longer matches the reference.",
            ));
        }
        let result = self.query_position(
            Path::new(&reference.file),
            reference.start,
            operation,
            limit,
            cancellation,
        )?;
        let actual = result.target.as_ref().and_then(|t| t.target_ref.as_ref());
        if actual != Some(reference) {
            return Err(SourceQueryError::new(
                "stale_target",
                "The checked declaration no longer matches the target reference.",
            ));
        }
        Ok(result)
    }
}

impl TargetReference {
    /// Checks untrusted reference structure before loading any recorded context.
    pub fn validate(&self) -> Result<(), SourceQueryError> {
        let digest = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        let path = |s: &str| {
            Path::new(s).is_absolute() && identity_path(Path::new(s)).is_ok_and(|p| p == s)
        };
        if self.version != 1
            || self.length == 0
            || self.start.checked_add(self.length).is_none()
            || self.qualified_name.trim().is_empty()
            || !digest(&self.snapshot_id)
            || !digest(&self.analysis_identity)
            || !digest(&self.file_sha256)
            || !path(&self.file)
            || !path(&self.context.input)
            || !path(&self.context.root)
            || self.owner.as_ref().is_some_and(|s| !path(s))
            || self
                .context
                .standard_library
                .as_ref()
                .is_some_and(|s| !path(s))
            || self.context.projects.iter().any(|s| !path(s))
            || self.context.projects.windows(2).any(|p| p[0] >= p[1])
            || self.context.policy != super::QueryPolicy::CheckedContext
        {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Malformed or unsupported target reference.",
            ));
        }
        Ok(())
    }
}

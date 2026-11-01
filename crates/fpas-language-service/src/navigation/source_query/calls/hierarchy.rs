//! Snapshot-bound owners include anonymous closures and executable declaration expressions.

use super::*;
use crate::{ScopeContext, SymbolKind};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

/// An exact source owner in a complete immutable semantic context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallableReference {
    /// Encoding version, currently one.
    pub version: u32,
    /// Frozen scope reconstruction information.
    pub context: ScopeContext,
    /// Exact source and membership digest.
    pub snapshot_id: String,
    /// Shared semantic build identity.
    pub analysis_identity: String,
    /// Exact physical owner; data must be re-resolved before use.
    pub callable: CallableIdentity,
}

impl CallableReference {
    /// Rejects malformed reconstruction data before an adapter loads its recorded context.
    pub fn validate(&self) -> Result<(), SourceQueryError> {
        let digest = |s: &str| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        let path = |s: &str| {
            Path::new(s).is_absolute() && crate::identity_path(Path::new(s)).is_ok_and(|p| p == s)
        };
        if self.version != 1
            || !digest(&self.snapshot_id)
            || !digest(&self.analysis_identity)
            || self.callable.file.as_deref().is_none_or(|s| !path(s))
            || self.callable.start.is_none()
            || self.callable.qualified_name.trim().is_empty()
            || self.callable.kind == CallableKind::Builtin
            || !path(&self.context.input)
            || !path(&self.context.root)
            || self.context.policy != crate::QueryPolicy::CheckedContext
            || self.context.kind == crate::ScopeKind::Folder
            || self
                .context
                .standard_library
                .as_ref()
                .is_some_and(|s| !path(s))
            || self.context.projects.iter().any(|s| !path(s))
            || self.context.projects.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Malformed callable reference.",
            ));
        }
        Ok(())
    }
}

impl SourceSession {
    /// Selects a referenced routine, or the innermost body containing the requested position.
    pub fn callable_at(
        &mut self,
        path: &Path,
        offset: usize,
        cancellation: &CancellationToken,
    ) -> Result<Option<Arc<SourceCallable>>, SourceQueryError> {
        let selected = self.query_position(
            path,
            offset,
            SourceQueryOperation::Definition,
            1,
            cancellation,
        )?;
        if !selected.reasons.is_empty() {
            return Err(incomplete());
        }
        let named = selected.target.as_ref().filter(|t| {
            matches!(
                t.kind,
                Some(
                    SymbolKind::Function
                        | SymbolKind::Procedure
                        | SymbolKind::Method
                        | SymbolKind::Unit
                        | SymbolKind::Program
                )
            )
        });
        let mut result: Option<Arc<SourceCallable>> = None;
        let selected_path = named
            .and_then(|t| t.snapshot.as_ref().map(|s| s.path()))
            .unwrap_or(path);
        let coverage = self.visit_call_inventory(
            Some(selected_path),
            cancellation,
            &mut |_| Ok(()),
            &mut |owner| {
                let eligible = if let Some(named) = named {
                    matches_target(owner, named)
                } else {
                    owner.snapshot.as_ref().is_some_and(|s| s.path() == path)
                        && owner
                            .span
                            .is_some_and(|span| span.offset() <= offset && offset < span.end())
                };
                if eligible
                    && result.as_ref().is_none_or(|current| {
                        owner.span.map(|s| s.length()) < current.span.map(|s| s.length())
                    })
                {
                    result = Some(Arc::clone(owner));
                }
            },
        )?;
        if !coverage.reasons.is_empty() {
            return Err(incomplete());
        }
        Ok(result)
    }

    /// Creates data for a previously resolved physical callable in this session.
    #[must_use]
    pub fn callable_reference(&self, callable: &SourceCallable) -> CallableReference {
        CallableReference {
            version: 1,
            context: self.context().clone(),
            snapshot_id: self.snapshot().id.clone(),
            analysis_identity: self.snapshot().analysis_identity.clone(),
            callable: callable.identity.clone(),
        }
    }

    /// Revalidates an untrusted hierarchy owner before collecting its direct source relationships.
    pub fn query_callable(
        &mut self,
        reference: &CallableReference,
        incoming: bool,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<CallRelations, SourceQueryError> {
        reference.validate()?;
        if reference.version != 1
            || reference.callable.file.is_none()
            || reference.callable.start.is_none()
            || !(1..=10000).contains(&limit)
        {
            return Err(SourceQueryError::new(
                "invalid_request",
                "Invalid callable reference or limit.",
            ));
        }
        if &reference.context != self.context() {
            return Err(SourceQueryError::new(
                "stale_target",
                "The callable's source context has changed.",
            ));
        }
        if reference.analysis_identity != self.snapshot().analysis_identity {
            return Err(SourceQueryError::new(
                "stale_target",
                "The callable's semantic implementation has changed.",
            ));
        }
        if reference.snapshot_id != self.snapshot().id {
            return Err(SourceQueryError::new(
                "stale_target",
                "The callable's input bytes or scope membership have changed.",
            ));
        }
        let selected_path = reference.callable.file.as_deref().map(Path::new);
        let mut found = false;
        let coverage = self.visit_call_inventory(
            selected_path,
            cancellation,
            &mut |_| Ok(()),
            &mut |owner| {
                found |= owner.identity == reference.callable;
            },
        )?;
        if !coverage.reasons.is_empty() {
            return Err(incomplete());
        }
        if !found {
            return Err(SourceQueryError::new(
                "stale_target",
                "The callable no longer matches a checked source owner.",
            ));
        }
        let (relations, _) = self.collect_relations(
            selected_path,
            incoming,
            limit,
            BTreeSet::new(),
            |owner| owner.identity == reference.callable,
            cancellation,
        )?;
        Ok(relations)
    }
}

fn incomplete() -> SourceQueryError {
    SourceQueryError::new(
        "analysis_incomplete",
        "Call hierarchy requires a completely checked source context.",
    )
}

//! Whole-context before/after checks shared by every source-edit adapter.

use super::{CheckedChanges, SourceChange, SourceEdit};
use crate::{CancellationToken, LanguageService, SourceQueryError, SourceVersion};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

const MAX_PREPARATION_BYTES: usize = 128 * 1024 * 1024;

impl LanguageService {
    /// Assembles exact replacements and checks every selected consumer against all overlays.
    /// This method never writes a file and preserves the caller's document store.
    pub fn validate_source_edits(
        &mut self,
        selected: &Path,
        edits: &[SourceEdit],
        cancellation: &CancellationToken,
    ) -> Result<CheckedChanges, SourceQueryError> {
        self.checked_edit_overlay(selected, edits, cancellation)
            .map(|(changes, _)| changes)
    }

    /// Retains the checked speculative service for transformations that must compare bindings.
    /// The live service and its document versions remain unchanged.
    pub(crate) fn checked_edit_overlay(
        &mut self,
        selected: &Path,
        edits: &[SourceEdit],
        cancellation: &CancellationToken,
    ) -> Result<(CheckedChanges, LanguageService), SourceQueryError> {
        cancellation.check()?;
        let mut paths = self
            .workspace()
            .projects()
            .iter()
            .flat_map(|p| p.all_source_paths())
            .collect::<BTreeSet<_>>();
        paths.insert(crate::document::normalized_path(selected));
        paths.extend(edits.iter().map(|e| e.path.clone()));
        let mut candidate = self.fork_for_edits();
        let mut inputs = Vec::new();
        let mut retained = 0usize;
        for path in &paths {
            cancellation.check()?;
            let snapshot = self.snapshot(path)?;
            retained = retained.saturating_add(snapshot.source().len());
            check_budget(retained)?;
            if !candidate.documents().is_open(snapshot.path()) {
                candidate
                    .documents_mut()
                    .open_document(snapshot.path(), 1, snapshot.source())?;
            }
            inputs.push(snapshot);
        }
        check_context(&mut candidate, &paths, cancellation)?;
        let mut grouped = BTreeMap::<_, Vec<&SourceEdit>>::new();
        for edit in edits {
            cancellation.check()?;
            if !crate::workspace::path_containment::contains(self.workspace().root(), &edit.path)
                || self.is_editor_api_source(&edit.path)
                || self.standard_library_root().is_some_and(|root| {
                    crate::workspace::path_containment::contains(root, &edit.path)
                })
            {
                return Err(SourceQueryError::new(
                    "outside_edit_scope",
                    "Affected sources must be inside the selected root and outside the standard library.",
                ));
            }
            let current = candidate.snapshot(&edit.path)?;
            if current.source() != edit.snapshot.source() {
                return Err(SourceQueryError::new(
                    "source_changed",
                    "An edit was prepared against an older source snapshot.",
                ));
            }
            let original = edit
                .snapshot
                .source()
                .get(edit.range.offset()..edit.range.end())
                .ok_or_else(|| {
                    SourceQueryError::new(
                        "invalid_edit",
                        "An edit range is outside its UTF-8 source.",
                    )
                })?;
            if original != edit.new_text {
                grouped
                    .entry(current.path().to_path_buf())
                    .or_default()
                    .push(edit);
            }
        }
        let mut changes = Vec::new();
        let mut checked_edits = Vec::new();
        for (path, mut edits) in grouped {
            edits.sort_by_key(|e| std::cmp::Reverse(e.range.offset()));
            let original = Arc::clone(&edits[0].snapshot);
            let mut replacement = original.source().to_owned();
            let mut boundary = replacement.len();
            let mut previous_start = None;
            for edit in edits {
                if edit.range.end() > boundary || previous_start == Some(edit.range.offset()) {
                    return Err(SourceQueryError::new(
                        "invalid_edit",
                        "Overlapping source edits cannot be applied.",
                    ));
                }
                retained = retained.saturating_add(edit.new_text.len());
                check_budget(retained)?;
                replacement.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
                boundary = edit.range.offset();
                previous_start = Some(boundary);
                checked_edits.push(edit.clone());
            }
            retained = retained.saturating_add(replacement.len());
            check_budget(retained)?;
            let version = candidate
                .documents()
                .open_snapshot(&path)
                .and_then(|s| match s.version() {
                    SourceVersion::Editor(v) => v.checked_add(1),
                    _ => Some(1),
                })
                .ok_or_else(|| {
                    SourceQueryError::new(
                        "stale_document",
                        "The editor document version cannot advance.",
                    )
                })?;
            candidate
                .documents_mut()
                .open_document(&path, version, replacement.as_str())?;
            changes.push(SourceChange {
                path,
                original,
                replacement,
            });
        }
        check_context(&mut candidate, &paths, cancellation)?;
        checked_edits.sort_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then(a.range.offset().cmp(&b.range.offset()))
        });
        Ok((
            CheckedChanges {
                inputs,
                edits: checked_edits,
                changes,
            },
            candidate,
        ))
    }
}

fn check_budget(bytes: usize) -> Result<(), SourceQueryError> {
    if bytes > MAX_PREPARATION_BYTES {
        Err(SourceQueryError::new(
            "plan_limit",
            "Original and replacement bytes exceed the 128 MiB preparation limit.",
        ))
    } else {
        Ok(())
    }
}

fn check_context(
    service: &mut LanguageService,
    paths: &BTreeSet<std::path::PathBuf>,
    cancellation: &CancellationToken,
) -> Result<(), SourceQueryError> {
    // Check each selected consumer in its own import/export allocation. Resolving a physical
    // file through its first owner would silently miss errors in another selected project.
    let discovery = crate::navigation::checked_source_context(service, cancellation)?;
    let mut missing = !discovery.reasons.is_empty();
    let mut diagnostics = discovery
        .diagnostics
        .into_iter()
        .filter(|d| {
            d.diagnostic.is_error()
                || d.diagnostic.code == fpas_diagnostics::codes::PROJECT_PROGRAM_SOURCE_SKIPPED
        })
        .collect::<Vec<_>>();
    let checked = discovery
        .batches
        .iter()
        .flat_map(|batch| batch.project.all_source_paths())
        .collect::<BTreeSet<_>>();
    for path in paths {
        cancellation.check()?;
        if checked.contains(path) || service.is_editor_api_source(path) {
            continue;
        }
        for analysis in service.source_analyses(path)? {
            if service.is_editor_api_source(analysis.snapshot().path()) {
                continue;
            }
            missing |= analysis.semantic().is_none();
            diagnostics.extend(
                analysis
                    .diagnostics()
                    .iter()
                    .filter(|d| d.is_error())
                    .map(|d| {
                        fpas_diagnostics::FileDiagnostic::new(
                            d.clone(),
                            Some(analysis.snapshot().path().to_path_buf()),
                        )
                        .with_source(analysis.snapshot().source())
                    }),
            );
        }
    }
    diagnostics.sort_by(|a, b| {
        a.path.cmp(&b.path).then(
            a.diagnostic
                .span
                .map(|s| s.offset())
                .cmp(&b.diagnostic.span.map(|s| s.offset())),
        )
    });
    diagnostics.dedup();
    if missing || !diagnostics.is_empty() {
        return Err(SourceQueryError {
            code: "analysis_incomplete",
            message: "Every selected source and consumer must check before and after the edit."
                .into(),
            diagnostics,
        });
    }
    Ok(())
}

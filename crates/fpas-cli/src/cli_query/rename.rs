//! Read-only rename planning, edit serialization, and semantic validation of the result.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_diagnostics::codes::{CLI_ARGUMENTS_INVALID, CLI_INPUT_UNSUPPORTED};
use fpas_language_service::{RenameEdit, RenameError};
use serde::Serialize;

use super::publication::{self, Change, Plan};
use super::{
    Context, Failure, Location, PreparedQuery, Range, check_analysis, display_path, source_range,
};
use crate::cli_input::QueryCliConfig;
use crate::cli_output::CliFailure;

#[derive(Serialize)]
pub(super) struct Report {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    query: &'static str,
    position: Location,
    new_name: String,
    applied: bool,
    edits: Vec<Edit>,
}

#[derive(Serialize)]
struct Edit {
    path: String,
    range: Range,
    old_text: String,
    new_text: String,
}

pub(super) fn prepare(
    config: &QueryCliConfig,
    mut context: Context,
) -> Result<PreparedQuery, Failure> {
    let new_name = config.new_name.as_deref().ok_or_else(|| {
        CliFailure::new(CLI_ARGUMENTS_INVALID, "Rename requires a replacement name.")
    })?;
    let mut paths = BTreeSet::from([context.snapshot.path().to_path_buf()]);
    for project in context.service.workspace().projects() {
        paths.extend(project.source_files().iter().cloned());
        paths.extend(project.main().map(Path::to_path_buf));
    }
    // Pin input snapshots before asking the shared service for ranges. The later overlay
    // check must use these same sources, and publication refuses changed disk contents.
    let mut snapshots = BTreeMap::new();
    for path in &paths {
        let snapshot = context.service.snapshot(path)?;
        if snapshot.path() == context.snapshot.path()
            && snapshot.source() != context.snapshot.source()
        {
            return Err(CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                "The selected source changed during rename; retry against the saved sources.",
            )
            .into());
        }
        snapshots.insert(snapshot.path().to_path_buf(), snapshot);
    }
    let paths = snapshots.keys().cloned().collect::<Vec<_>>();
    let inputs = snapshots.into_values().collect::<Vec<_>>();
    for snapshot in &inputs {
        context
            .service
            .documents_mut()
            .open_document(snapshot.path(), 1, snapshot.source())?;
    }
    for path in &paths {
        check_analysis(&mut context.service, path)?;
    }
    let edits = context
        .service
        .rename(context.snapshot.path(), context.offset, new_name)
        .map_err(|error| match error {
            RenameError::Service(error) => Failure::from(error),
            error => CliFailure::new(CLI_INPUT_UNSUPPORTED, error.to_string()).into(),
        })?
        .value;
    let root = canonical(context.service.workspace().root())?;
    let library = context
        .standard_library
        .as_deref()
        .map(canonical)
        .transpose()?;
    let mut grouped: BTreeMap<PathBuf, Vec<RenameEdit>> = BTreeMap::new();
    let mut report_edits = Vec::with_capacity(edits.len());
    for edit in edits {
        let path = canonical(&edit.path)?;
        if !path.starts_with(&root)
            || library
                .as_ref()
                .is_some_and(|library| path.starts_with(library))
        {
            return Err(CliFailure::new(CLI_INPUT_UNSUPPORTED, format!(
                "Cannot rename `{}`: affected sources must be inside the selected root and outside the standard library.", edit.path.display()
            )).into());
        }
        let old_text = edit
            .snapshot
            .source()
            .get(edit.range.offset()..edit.range.end())
            .ok_or_else(|| {
                CliFailure::new(
                    CLI_INPUT_UNSUPPORTED,
                    "Rename returned an invalid source range.",
                )
            })?;
        if old_text == edit.new_text {
            continue;
        }
        report_edits.push(Edit {
            path: display_path(&edit.path),
            range: source_range(&edit.snapshot, edit.range)?,
            old_text: old_text.to_owned(),
            new_text: edit.new_text.clone(),
        });
        grouped.entry(path).or_default().push(edit);
    }
    let mut changes = Vec::with_capacity(grouped.len());
    for (path, mut edits) in grouped {
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.offset()));
        let original = Arc::clone(&edits[0].snapshot);
        let mut replacement = original.source().to_owned();
        let mut boundary = replacement.len();
        for edit in edits {
            if edit.snapshot.source() != original.source() || edit.range.end() > boundary {
                return Err(CliFailure::new(
                    CLI_INPUT_UNSUPPORTED,
                    "Rename returned overlapping edits or inconsistent source snapshots.",
                )
                .into());
            }
            replacement.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
            boundary = edit.range.offset();
        }
        changes.push(Change {
            path,
            original,
            replacement,
        });
    }
    for change in &changes {
        context.service.documents_mut().open_document(
            change.original.path(),
            2,
            change.replacement.as_str(),
        )?;
    }
    for path in &paths {
        check_analysis(&mut context.service, path)?;
    }
    let plan = Plan { inputs, changes };
    plan.verify()?;
    Ok(PreparedQuery {
        report: super::Report::Rename(Report {
            schema_version: 1,
            query: "rename",
            position: Location {
                path: display_path(context.snapshot.path()),
                line: config.line,
                column: config.column,
            },
            new_name: new_name.to_owned(),
            applied: !config.dry_run,
            edits: report_edits,
        }),
        publication: (!config.dry_run).then_some(plan),
    })
}

fn canonical(path: &Path) -> Result<PathBuf, Failure> {
    fs::canonicalize(path).map_err(|error| publication::failure(path, error))
}

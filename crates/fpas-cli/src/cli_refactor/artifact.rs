//! Atomic immutable preview artifacts that cannot join or alias the analysis input set.

use super::super::{protocol::Report, report};
use super::{RefactorAction, RefactorCliConfig, failure, reload};
use crate::cli_input::QuerySelector;
use fpas_language_service::{RefactorPlan, SourceQueryError, identity_path};
use std::{fs, path::Path, time::Instant};

pub(super) fn write(
    output: &Path,
    config: &RefactorCliConfig,
    plan: &RefactorPlan,
    report: &mut Report,
    deadline: Instant,
) -> Result<(), SourceQueryError> {
    let session = reload(plan, deadline)?;
    let physical = if output.exists() {
        fs::canonicalize(output)
    } else {
        output
            .parent()
            .zip(output.file_name())
            .ok_or_else(|| std::io::Error::other("Output requires an existing parent directory."))
            .and_then(|(parent, name)| fs::canonicalize(parent).map(|p| p.join(name)))
    }
    .map_err(|e| failure("artifact_output", e.to_string()))?;
    let identity =
        identity_path(&physical).map_err(|e| failure("artifact_output", e.to_string()))?;
    let reference = match &config.action {
        RefactorAction::Rename {
            selector: QuerySelector::Reference(path),
            ..
        } => Some(path),
        RefactorAction::AddParameter {
            selector: QuerySelector::Reference(path),
            ..
        } => Some(path),
        _ => None,
    };
    if plan.before.files.iter().any(|f| f.path == identity)
        || plan
            .before
            .aliases
            .iter()
            .any(|(_, target)| target == &identity)
        || reference.is_some_and(|p| fs::canonicalize(p).is_ok_and(|p| p == physical))
    {
        return Err(failure(
            "artifact_alias",
            "Plan output aliases an analysis or reference input.",
        ));
    }
    for glob in &plan.before.globs {
        if fpas_project::would_select_path(
            Path::new(&glob.root),
            &glob.pattern,
            &glob.exclude,
            &physical,
        )
        .map_err(|e| failure("artifact_output", e.to_string()))?
        {
            return Err(failure(
                "artifact_membership",
                "Plan output would change recorded source membership.",
            ));
        }
    }
    let bytes = report::finish(report);
    if report.exit_code != 0 {
        return Err(failure(
            "plan_limit",
            "The exact preview exceeds the public JSON limit.",
        ));
    }
    if !session.verify()? {
        return Err(failure(
            "stale_plan",
            "Inputs changed before artifact publication.",
        ));
    }
    if Instant::now() >= deadline {
        return Err(failure(
            "execution_timeout",
            "Preparation deadline expired before artifact publication.",
        ));
    }
    match crate::oneshot::read_bounded(&physical, report::MAX_JSON) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => {
            return Err(failure(
                "artifact_exists",
                "A different artifact already exists at the output path.",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(failure("artifact_output", error.to_string())),
    }
    match crate::cli_failure_dump::publish_report(&physical, &bytes) {
        Ok(()) => Ok(()),
        Err(error)
            if error.kind() == std::io::ErrorKind::AlreadyExists
                && crate::oneshot::read_bounded(&physical, report::MAX_JSON)
                    .is_ok_and(|existing| existing == bytes) =>
        {
            Ok(())
        }
        Err(error) => Err(failure("artifact_output", error.to_string())),
    }
}

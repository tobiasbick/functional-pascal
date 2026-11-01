//! Surviving coordinator: stage all files, then publish or recover without a read deadline.

use super::super::{protocol::*, report};
use super::{failure, reload};
use atomic_write_file::AtomicWriteFile;
use fpas_language_service::{
    RefactorPlan, RefactorStatus, SourceChange, SourceQueryError, identity_path, source_digest,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path, time::Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(in crate::cli_query) struct Publication {
    pub status: String,
    pub paths: Vec<PublishedPath>,
    pub publication_error: Option<String>,
    pub recovery_errors: Vec<RecoveryError>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(in crate::cli_query) struct PublishedPath {
    pub path: String,
    pub state: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(in crate::cli_query) struct RecoveryError {
    pub path: String,
    pub error: String,
}

pub(super) fn publish(
    plan: &RefactorPlan,
    deadline: Instant,
    report: &mut Report,
) -> Result<(), SourceQueryError> {
    let mut session = reload(plan, deadline)?;
    let mut changes = Vec::new();
    let mut retained = 0usize;
    for before in &plan.before.files {
        let edits = plan
            .edits
            .iter()
            .filter(|e| e.path == before.path)
            .collect::<Vec<_>>();
        if edits.is_empty() {
            continue;
        }
        let original = session.service().snapshot(Path::new(&before.path))?;
        let mut replacement = original.source().to_owned();
        for edit in edits.iter().rev() {
            if replacement.get(edit.start..edit.start + edit.length) != Some(edit.old_text.as_str())
            {
                return Err(failure(
                    "stale_plan",
                    "Original bytes changed after recipe validation.",
                ));
            }
            replacement.replace_range(edit.start..edit.start + edit.length, &edit.new_text);
        }
        if plan
            .after
            .files
            .iter()
            .find(|f| f.path == before.path)
            .is_none_or(|after| {
                after.bytes != replacement.len()
                    || after.sha256 != source_digest(replacement.as_bytes())
            })
        {
            return Err(failure(
                "invalid_plan",
                "Exact replacements disagree with the expected inventory.",
            ));
        }
        retained = retained
            .saturating_add(original.source().len())
            .saturating_add(replacement.len());
        if retained > 128 * 1024 * 1024 {
            return Err(failure(
                "plan_limit",
                "Replacement buffers exceed the preparation limit.",
            ));
        }
        changes.push(SourceChange {
            path: original.path().into(),
            original,
            replacement,
        });
    }
    let inventory = session.inventory().clone();
    if !session.verify()? {
        return Err(failure("stale_plan", "Inputs changed before staging."));
    }
    drop(session);
    let mut publication = Publication {
        status: "failed".into(),
        paths: changes
            .iter()
            .map(|c| {
                Ok(PublishedPath {
                    path: identity_path(&c.path)
                        .map_err(|e| failure("unsupported_path_identity", e.to_string()))?,
                    state: "unchanged".into(),
                    error: None,
                })
            })
            .collect::<Result<Vec<_>, SourceQueryError>>()?,
        publication_error: None,
        recovery_errors: Vec::new(),
    };
    reserve_report(report, &publication)?;
    let outcome = publish_changes(&changes, &inventory, deadline, &mut publication);
    if let Some(Payload::Refactor {
        status,
        publication: result,
        ..
    }) = &mut report.result
    {
        if outcome.is_ok() {
            *status = RefactorStatus::Applied;
        }
        *result = Some(publication);
    }
    outcome
}

fn publish_changes(
    changes: &[SourceChange],
    inventory: &fpas_project::source_inputs::InputInventory,
    deadline: Instant,
    result: &mut Publication,
) -> Result<(), SourceQueryError> {
    let mut staged = Vec::with_capacity(changes.len());
    for (index, change) in changes.iter().enumerate() {
        let prepare = || -> std::io::Result<AtomicWriteFile> {
            let permissions = fs::metadata(&change.path)?.permissions();
            if permissions.readonly() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Source is read-only.",
                ));
            }
            let mut file = AtomicWriteFile::open(&change.path)?;
            file.write_all(change.replacement.as_bytes())?;
            file.as_file().set_permissions(permissions)?;
            Ok(file)
        };
        match prepare() {
            Ok(file) => staged.push(file),
            Err(error) => {
                result.paths[index].error = Some(message(&error));
                return Err(publication_failure(
                    result,
                    "Cannot stage all source replacements.",
                ));
            }
        }
    }
    match inventory.verify() {
        Ok(true) => {}
        Ok(false) => {
            return Err(publication_failure(
                result,
                "Inputs changed before the first source replacement.",
            ));
        }
        Err(error) => return Err(publication_failure(result, &message(&error))),
    }
    if Instant::now() >= deadline {
        return Err(failure(
            "execution_timeout",
            "Preparation deadline expired before the first source replacement.",
        ));
    }
    for (index, file) in staged.into_iter().enumerate() {
        let change = &changes[index];
        let commit = match fs::read(&change.path) {
            Ok(bytes) if bytes == change.original.source().as_bytes() => file.commit(),
            Ok(_) => Err(std::io::Error::other("Source changed before replacement.")),
            Err(error) => Err(error),
        };
        if let Err(error) = commit {
            result.paths[index].error = Some(message(&error));
            result.publication_error = Some(message(&error));
            for (index, change) in changes[..=index].iter().enumerate().rev() {
                match recover(change) {
                    Ok(state) => result.paths[index].state = state.into(),
                    Err(error) => {
                        result.paths[index].state = "unresolved".into();
                        result.paths[index].error = Some(message(&error));
                        result.recovery_errors.push(RecoveryError {
                            path: result.paths[index].path.clone(),
                            error: message(&error),
                        });
                    }
                }
            }
            result.status = if result.recovery_errors.is_empty() {
                "restored"
            } else {
                "unresolved"
            }
            .into();
            return Err(failure(
                "publication_failed",
                "Source publication failed; inspect per-path recovery results.",
            ));
        }
        result.paths[index].state = "replaced".into();
    }
    result.status = "applied".into();
    Ok(())
}

fn recover(change: &SourceChange) -> std::io::Result<&'static str> {
    let current = fs::read(&change.path)?;
    if current == change.original.source().as_bytes() {
        return Ok("unchanged");
    }
    if current != change.replacement.as_bytes() {
        return Err(std::io::Error::other(
            "Concurrent newer text was preserved.",
        ));
    }
    crate::source_publication::write_source(&change.path, change.original.source())?;
    Ok("restored")
}

fn message(error: &impl std::fmt::Display) -> String {
    error.to_string().chars().take(512).collect()
}
fn publication_failure(result: &mut Publication, message: &str) -> SourceQueryError {
    result.publication_error = Some(message.into());
    failure("publication_failed", message)
}

fn reserve_report(report: &Report, publication: &Publication) -> Result<(), SourceQueryError> {
    // Reserve every path's worst ordinary recovery record before any source write. Exact
    // edits and path states must remain reportable even if publication later fails.
    let mut probe = report.clone();
    let mut publication = publication.clone();
    publication.status = "unresolved".into();
    // Control characters require six JSON bytes each, exceeding even four-byte Unicode.
    publication.publication_error = Some("\0".repeat(512));
    for path in &mut publication.paths {
        path.state = "unresolved".into();
        path.error = Some("\0".repeat(512));
        publication.recovery_errors.push(RecoveryError {
            path: path.path.clone(),
            error: "\0".repeat(512),
        });
    }
    if let Some(Payload::Refactor {
        publication: result,
        ..
    }) = &mut probe.result
    {
        *result = Some(publication);
    }
    let bytes = serde_json::to_vec(&probe).map_err(|e| failure("plan_limit", e.to_string()))?;
    if bytes.len().saturating_add(4096) > report::MAX_JSON {
        return Err(failure(
            "plan_limit",
            "The plan and publication/recovery records exceed the public report limit.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_restores_only_its_own_replacement_and_preserves_newer_text() {
        let directory = std::env::temp_dir().join(format!("fpas-recovery-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("recovery fixture");
        let path = directory.join("main.fpas");
        let before = "program Before; begin end.";
        let replacement = "program Replacement; begin end.";
        fs::write(&path, before).expect("original source");
        let mut service = fpas_language_service::LanguageService::new(
            fpas_language_service::WorkspaceContext::loose(&path),
        );
        let change = SourceChange {
            path: path.clone(),
            original: service.snapshot(&path).expect("original snapshot"),
            replacement: replacement.into(),
        };
        assert_eq!(recover(&change).expect("unchanged file"), "unchanged");
        fs::write(&path, replacement).expect("our replacement");
        assert_eq!(
            recover(&change).expect("restore own replacement"),
            "restored"
        );
        assert_eq!(fs::read_to_string(&path).expect("restored source"), before);
        let newer = "program Concurrent; begin end.";
        fs::write(&path, newer).expect("concurrent newer source");
        assert!(recover(&change).is_err());
        assert_eq!(fs::read_to_string(&path).expect("preserved source"), newer);
        fs::remove_dir_all(directory).expect("remove fixture");
    }
}

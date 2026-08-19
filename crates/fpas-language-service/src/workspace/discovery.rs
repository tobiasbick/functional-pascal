//! Initial and document-driven FPAS manifest discovery.

use std::path::{Path, PathBuf};

use fpas_project::discover_workspace_file;

use super::path_containment;
use super::{WorkspaceContext, WorkspaceIssue, WorkspaceKind};
use crate::document::normalized_path;

pub(super) fn discover_initial_context(input: &Path) -> WorkspaceContext {
    let start = directory_for(input);
    match discover_manifest(&start) {
        Ok(Some(path)) if has_extension(&path, "fpasworkspace") => {
            WorkspaceContext::load_workspace_manifest(&path)
        }
        Ok(Some(path)) => WorkspaceContext::load_project_manifest(&path),
        Ok(None) => WorkspaceContext::loose(input),
        Err(issue) => WorkspaceContext {
            root: input.to_path_buf(),
            manifest_path: None,
            kind: WorkspaceKind::Unavailable,
            projects: Vec::new(),
            issues: vec![issue],
        },
    }
}

pub(super) fn discover_source_context(
    root: &Path,
    source: &Path,
) -> Result<Option<WorkspaceContext>, WorkspaceIssue> {
    let root = directory_for(&normalized_path(root));
    let source = normalized_path(source);
    let bounded = path_containment::contains(&root, &source);
    let mut directory = directory_for(&source);

    loop {
        if let Some(context) = context_owning_source(&directory, &source)? {
            return Ok(Some(context));
        }
        if (bounded && path_containment::same(&directory, &root)) || !directory.pop() {
            return Ok(None);
        }
    }
}

fn context_owning_source(
    directory: &Path,
    source: &Path,
) -> Result<Option<WorkspaceContext>, WorkspaceIssue> {
    if !directory.is_dir() {
        return Ok(None);
    }
    if let Some(workspace_path) =
        discover_workspace_file(directory).map_err(|message| WorkspaceIssue {
            path: directory.to_path_buf(),
            message,
        })?
    {
        let context = WorkspaceContext::load_workspace_manifest(&normalized_path(&workspace_path));
        if let Some(result) = select_context(source, vec![context])? {
            return Ok(Some(result));
        }
    }

    let manifests = direct_project_manifests(directory)?;
    let contexts = manifests
        .iter()
        .map(|manifest| WorkspaceContext::load_project_manifest(manifest))
        .collect::<Vec<_>>();
    select_context(source, contexts)
}

fn select_context(
    source: &Path,
    contexts: Vec<WorkspaceContext>,
) -> Result<Option<WorkspaceContext>, WorkspaceIssue> {
    let mut owning = Vec::new();
    let mut consuming = Vec::new();
    let mut load_issues = Vec::new();
    for context in contexts {
        let owners = context
            .projects()
            .iter()
            .filter(|project| project.owns_source(source))
            .map(|project| project.manifest_path().to_path_buf())
            .collect::<Vec<_>>();
        if owners.is_empty() {
            if context
                .projects()
                .iter()
                .any(|project| project.contains_source(source))
            {
                consuming.push(context);
            } else {
                load_issues.extend(context.issues().iter().cloned());
            }
        } else {
            owning.push((owners, context));
        }
    }

    let owner_paths = owning
        .iter()
        .flat_map(|(owners, _)| owners.iter())
        .collect::<Vec<_>>();
    match owner_paths.len() {
        0 => {
            if let Some(issue) = load_issues.into_iter().next() {
                Err(issue)
            } else {
                Ok(None)
            }
        }
        1 => {
            let Some((_, mut context)) = owning.pop() else {
                return Ok(None);
            };
            for consumer in consuming {
                context.merge_discovered(consumer);
            }
            Ok(Some(context))
        }
        _ => Err(ambiguous_source_issue(source, &owner_paths)),
    }
}

fn ambiguous_source_issue(source: &Path, manifests: &[&PathBuf]) -> WorkspaceIssue {
    let mut names = manifests
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    WorkspaceIssue {
        path: source.to_path_buf(),
        message: format!(
            "Source belongs directly to multiple FPAS projects: {}.\n  help: Adjust `[sources]` so exactly one nearest project owns this file.",
            names.join(", ")
        ),
    }
}

fn discover_manifest(start: &Path) -> Result<Option<PathBuf>, WorkspaceIssue> {
    let mut directory = start.to_path_buf();
    loop {
        match discover_workspace_file(&directory) {
            Ok(Some(path)) => return Ok(Some(normalized_path(&path))),
            Ok(None) => {}
            Err(message) => {
                return Err(WorkspaceIssue {
                    path: directory,
                    message,
                });
            }
        }

        let mut projects = direct_project_manifests(&directory)?;
        match projects.len() {
            0 => {}
            1 => return Ok(projects.pop()),
            _ => {
                let names = projects
                    .iter()
                    .filter_map(|path| path.file_name())
                    .map(|name| name.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(WorkspaceIssue {
                    path: directory,
                    message: format!(
                        "Found multiple `.fpasprj` files while discovering editor context: {names}.\n  help: Open a source owned by the desired project or an explicit workspace manifest."
                    ),
                });
            }
        }

        if !directory.pop() {
            return Ok(None);
        }
    }
}

fn direct_project_manifests(directory: &Path) -> Result<Vec<PathBuf>, WorkspaceIssue> {
    let entries = std::fs::read_dir(directory).map_err(|error| WorkspaceIssue {
        path: directory.to_path_buf(),
        message: format!("Cannot inspect editor workspace directory: {error}"),
    })?;
    let mut projects = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && has_extension(path, "fpasprj"))
        .map(|path| normalized_path(&path))
        .collect::<Vec<_>>();
    projects.sort();
    Ok(projects)
}

fn directory_for(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

pub(super) fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn fixture(label: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        std::env::temp_dir().join(format!(
            "fpas-discovery-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[cfg(windows)]
    #[test]
    fn nonexistent_inside_document_with_different_case_stops_at_root() {
        let base = fixture("case-boundary");
        let root = base.join("Workspace");
        std::fs::create_dir_all(root.join("src")).expect("workspace directories");
        std::fs::write(base.join("broken.fpasprj"), "not valid TOML")
            .expect("outer invalid manifest");
        let differently_cased =
            PathBuf::from(root.to_string_lossy().to_ascii_lowercase()).join("src/missing.fpas");

        let discovered = discover_source_context(&root, &differently_cased);
        std::fs::remove_dir_all(&base).ok();
        assert!(matches!(discovered, Ok(None)), "{discovered:?}");
    }

    #[test]
    fn genuinely_outside_document_keeps_unbounded_discovery() {
        let base = fixture("outside-boundary");
        let root = base.join("workspace");
        let outside = base.join("outside");
        std::fs::create_dir_all(&root).expect("workspace directory");
        std::fs::create_dir_all(&outside).expect("outside directory");
        std::fs::write(outside.join("broken.fpasprj"), "not valid TOML")
            .expect("outside invalid manifest");

        let discovered = discover_source_context(&root, &outside.join("missing.fpas"));
        std::fs::remove_dir_all(&base).ok();
        assert!(
            discovered.is_err(),
            "outside discovery must remain unbounded"
        );
    }
}

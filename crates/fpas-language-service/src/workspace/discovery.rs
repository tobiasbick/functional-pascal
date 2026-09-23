//! Initial and document-driven FPAS manifest discovery.

use std::path::{Path, PathBuf};

use fpas_project::discover_workspace_file;

use super::path_containment;
use super::{WorkspaceContext, WorkspaceIssue, WorkspaceKind};
use crate::document::normalized_path;

enum DiscoveryError {
    Directory(WorkspaceIssue),
    Metadata(WorkspaceIssue),
}

impl DiscoveryError {
    fn into_issue(self) -> WorkspaceIssue {
        match self {
            Self::Directory(issue) | Self::Metadata(issue) => issue,
        }
    }
}

/// Discover the nearest project that owns an initial source or return a loose context.
pub(super) fn discover_initial_context(input: &Path) -> WorkspaceContext {
    discover_initial_context_with(input, |directory| context_owning_source(directory, input))
}

fn discover_initial_context_with(
    input: &Path,
    inspect: impl Fn(&Path) -> Result<Option<WorkspaceContext>, DiscoveryError>,
) -> WorkspaceContext {
    let start = directory_for(input);
    let mut directory = start.clone();
    loop {
        match inspect(&directory) {
            Ok(Some(context)) => return context,
            Ok(None) => {}
            Err(DiscoveryError::Directory(_)) if input.is_file() && directory != start => {
                return WorkspaceContext::loose(input);
            }
            Err(issue) => {
                return WorkspaceContext {
                    root: directory,
                    manifest_path: None,
                    kind: WorkspaceKind::Unavailable,
                    projects: Vec::new(),
                    issues: vec![issue.into_issue()],
                };
            }
        }
        if !directory.pop() {
            return WorkspaceContext::loose(input);
        }
    }
}

/// Resolves ownership inside the session root, retaining manifest errors on later requests.
pub(super) fn discover_source_context(
    root: &Path,
    source: &Path,
) -> Result<Option<WorkspaceContext>, WorkspaceIssue> {
    let root = directory_for(&normalized_path(root));
    let source = normalized_path(source);
    let bounded = path_containment::contains(&root, &source);
    let mut directory = directory_for(&source);

    loop {
        if let Some(context) =
            context_owning_source(&directory, &source).map_err(DiscoveryError::into_issue)?
        {
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
) -> Result<Option<WorkspaceContext>, DiscoveryError> {
    if !directory.is_dir() {
        return Ok(None);
    }
    let manifests = direct_project_manifests(directory)?;
    if let Some(workspace_path) = discover_workspace_file(directory)
        .map_err(|message| WorkspaceIssue {
            path: directory.to_path_buf(),
            message,
        })
        .map_err(DiscoveryError::Metadata)?
    {
        let context = WorkspaceContext::load_workspace_manifest(&normalized_path(&workspace_path));
        if let Some(result) =
            select_context(source, vec![context]).map_err(DiscoveryError::Metadata)?
        {
            return Ok(Some(result));
        }
    }

    let contexts = manifests
        .iter()
        .map(|manifest| WorkspaceContext::load_project_manifest(manifest))
        .collect::<Vec<_>>();
    select_context(source, contexts).map_err(DiscoveryError::Metadata)
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

fn direct_project_manifests(directory: &Path) -> Result<Vec<PathBuf>, DiscoveryError> {
    let directory_error = |error| {
        DiscoveryError::Directory(WorkspaceIssue {
            path: directory.to_path_buf(),
            message: format!("Cannot inspect editor workspace directory: {error}"),
        })
    };
    let mut projects = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(directory_error)? {
        let path = entry.map_err(directory_error)?.path();
        if path.is_file() && has_extension(&path, "fpasprj") {
            projects.push(normalized_path(&path));
        }
    }
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
#[allow(clippy::expect_used)]
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

    #[test]
    fn unreadable_ancestor_does_not_hide_readable_loose_source() {
        let base = fixture("unreadable-ancestor");
        let child = base.join("source");
        std::fs::create_dir_all(&child).expect("source directory");
        let source = child.join("loose.fpas");
        std::fs::write(&source, "program Loose; begin end.").expect("source file");
        let context = discover_initial_context_with(&source, |directory| {
            if directory == child {
                Ok(None)
            } else {
                Err(DiscoveryError::Directory(WorkspaceIssue {
                    path: directory.to_path_buf(),
                    message: "injected access denied".to_string(),
                }))
            }
        });
        assert_eq!(context.kind(), WorkspaceKind::Loose);
        assert!(context.issues().is_empty());
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn nearest_manifest_is_selected_before_ancestor_failure() {
        let base = fixture("nearest-manifest");
        let child = base.join("source");
        std::fs::create_dir_all(&child).expect("source directory");
        let manifest = child.join("app.fpasprj");
        std::fs::write(&manifest, "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n\n[sources]\ninclude = [\"main.fpas\"]\n")
            .expect("project manifest");
        let source = child.join("main.fpas");
        std::fs::write(&source, "program App; begin end.").expect("project source");
        let context = discover_initial_context_with(&source, |directory| {
            if directory == child {
                context_owning_source(directory, &source)
            } else {
                Err(DiscoveryError::Directory(WorkspaceIssue {
                    path: directory.to_path_buf(),
                    message: "injected access denied".to_string(),
                }))
            }
        });
        assert_eq!(context.kind(), WorkspaceKind::Project);
        assert_eq!(context.manifest_path(), Some(manifest.as_path()));
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn unrelated_ancestor_project_does_not_own_loose_source() {
        let base = fixture("unrelated-project");
        let child = base.join("source");
        std::fs::create_dir_all(&child).expect("source directory");
        let source = child.join("loose.fpas");
        std::fs::write(&source, "program Loose; begin end.").expect("loose source");
        std::fs::write(base.join("other.fpas"), "unit Other;").expect("unrelated source");
        std::fs::write(base.join("other.fpasprj"), "[project]\nname = \"other\"\nkind = \"library\"\n\n[sources]\ninclude = [\"other.fpas\"]\n")
            .expect("unrelated project");

        let context = discover_initial_context(&source);
        assert_eq!(context.kind(), WorkspaceKind::Loose);
        std::fs::remove_dir_all(base).ok();
    }
}

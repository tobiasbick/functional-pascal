//! Bounded manifest catalog for one editor folder.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::discovery::has_extension;
use super::{WorkspaceContext, WorkspaceKind};
use crate::CancellationToken;
use crate::document::normalized_path;

pub(super) fn load_folder(
    root: &Path,
    cancellation: &CancellationToken,
) -> Result<WorkspaceContext, crate::LanguageServiceError> {
    let root = normalized_path(root);
    let manifests = collect_manifests(&root, cancellation)?;
    let mut projects = Vec::new();
    let mut issues = Vec::new();
    let mut loaded_manifests = BTreeSet::new();

    for manifest in manifests {
        cancellation.check()?;
        let context = if has_extension(&manifest, "fpasworkspace") {
            WorkspaceContext::load_workspace_manifest(&manifest)
        } else {
            WorkspaceContext::load_project_manifest(&manifest)
        };
        for project in context.projects {
            if loaded_manifests.insert(project.manifest_path().to_path_buf()) {
                projects.push(project);
            }
        }
        for issue in context.issues {
            if !issues.contains(&issue) {
                issues.push(issue);
            }
        }
    }
    projects.sort_by(|left, right| left.manifest_path().cmp(right.manifest_path()));
    issues.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(WorkspaceContext {
        root,
        manifest_path: None,
        kind: WorkspaceKind::Folder,
        projects,
        issues,
    })
}

fn collect_manifests(
    root: &Path,
    cancellation: &CancellationToken,
) -> Result<Vec<PathBuf>, crate::LanguageServiceError> {
    let mut pending = vec![root.to_path_buf()];
    let mut manifests = Vec::new();
    while let Some(directory) = pending.pop() {
        cancellation.check()?;
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                return Err(crate::LanguageServiceError::analysis(
                    &directory,
                    format!("Cannot inspect editor workspace directory: {error}"),
                ));
            }
        };
        let mut children = Vec::new();
        for entry in entries.flatten() {
            cancellation.check()?;
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if !ignored_directory(&path) {
                    children.push(path);
                }
            } else if file_type.is_file()
                && (has_extension(&path, "fpasprj") || has_extension(&path, "fpasworkspace"))
            {
                manifests.push(normalized_path(&path));
            }
        }
        children.sort();
        children.reverse();
        pending.extend(children);
    }
    manifests.sort();
    manifests.dedup();
    Ok(manifests)
}

fn ignored_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                ".git" | ".vscode-test" | "node_modules" | "target"
            )
        })
}

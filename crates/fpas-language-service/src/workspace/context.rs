//! Manifest-backed workspace context.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fpas_project::{LoadedProject, load_project, load_standard_library_project, load_workspace};

use super::ProjectContext;
use super::catalog::load_folder;
use super::discovery::{discover_initial_context, discover_source_context, has_extension};
use crate::CancellationToken;
use crate::document::normalized_path;

/// Broad source ownership mode selected for a language-service session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceKind {
    /// No usable manifest was found; documents are analyzed independently.
    Loose,
    /// One `.fpasprj` project was loaded.
    Project,
    /// One `.fpasworkspace` and its usable member projects were loaded.
    Workspace,
    /// An editor folder contains lazily discovered nested FPAS projects.
    Folder,
    /// A requested manifest existed but could not be loaded.
    Unavailable,
}

/// A recoverable context-loading problem that does not panic or terminate the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIssue {
    /// Manifest or discovery path associated with the problem.
    pub path: PathBuf,
    /// Actionable error returned by the authoritative project loader.
    pub message: String,
}

/// Loaded editor context with recoverable manifest problems.
#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    pub(super) root: PathBuf,
    pub(super) manifest_path: Option<PathBuf>,
    pub(super) kind: WorkspaceKind,
    pub(super) projects: Vec<ProjectContext>,
    pub(super) issues: Vec<WorkspaceIssue>,
}

impl WorkspaceContext {
    /// Loads an explicit source, directory, project, or workspace path.
    ///
    /// Invalid or absent metadata becomes [`WorkspaceIssue`] state instead of a constructor error.
    #[must_use]
    pub fn load(input: &Path) -> Self {
        Self::load_with_cancellation(input, &CancellationToken::new()).unwrap_or_else(|error| {
            Self {
                root: normalized_path(input),
                manifest_path: None,
                kind: WorkspaceKind::Unavailable,
                projects: Vec::new(),
                issues: vec![WorkspaceIssue {
                    path: normalized_path(input),
                    message: error.to_string(),
                }],
            }
        })
    }

    /// Loads context while observing a cooperative cancellation signal.
    pub fn load_with_cancellation(
        input: &Path,
        cancellation: &CancellationToken,
    ) -> Result<Self, crate::LanguageServiceError> {
        let input = normalized_path(input);
        cancellation.check()?;
        let context = if has_extension(&input, "fpasprj") {
            Self::load_project_manifest(&input)
        } else if has_extension(&input, "fpasworkspace") {
            Self::load_workspace_manifest(&input)
        } else if input.is_dir() {
            return load_folder(&input, cancellation);
        } else {
            discover_initial_context(&input)
        };
        cancellation.check()?;
        Ok(context)
    }

    /// Creates a loose-file context without project discovery.
    #[must_use]
    pub fn loose(root: &Path) -> Self {
        Self {
            root: normalized_path(root),
            manifest_path: None,
            kind: WorkspaceKind::Loose,
            projects: Vec::new(),
            issues: Vec::new(),
        }
    }

    /// Returns the normalized source or discovery root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the loaded context category.
    #[must_use]
    pub fn kind(&self) -> WorkspaceKind {
        self.kind
    }

    /// Returns the selected project or workspace manifest path.
    #[must_use]
    pub fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }

    /// Returns every successfully loaded project.
    #[must_use]
    pub fn projects(&self) -> &[ProjectContext] {
        &self.projects
    }

    /// Returns recoverable discovery and loading issues.
    #[must_use]
    pub fn issues(&self) -> &[WorkspaceIssue] {
        &self.issues
    }

    /// Finds the loaded project that contains a source, preferring direct ownership.
    #[must_use]
    pub fn project_for_source(&self, path: &Path) -> Option<&ProjectContext> {
        let path = normalized_path(path);
        self.projects
            .iter()
            .find(|project| project.owns_source(&path))
            .or_else(|| {
                self.projects
                    .iter()
                    .find(|project| project.contains_source(&path))
            })
    }

    pub(crate) fn discover_project_for_source(
        &mut self,
        path: &Path,
    ) -> Result<(), WorkspaceIssue> {
        let owners = self
            .projects
            .iter()
            .filter(|project| project.owns_source(path))
            .map(|project| project.manifest_path())
            .collect::<Vec<_>>();
        if owners.len() > 1 {
            return Err(ambiguous_source_issue(path, &owners));
        }
        if owners.len() == 1 {
            return Ok(());
        }
        let Some(context) = discover_source_context(&self.root, path)? else {
            return Ok(());
        };
        self.merge_discovered(context);
        Ok(())
    }

    pub(crate) fn reload_folder(
        &mut self,
        cancellation: &CancellationToken,
    ) -> Result<BTreeSet<PathBuf>, crate::LanguageServiceError> {
        if self.kind != WorkspaceKind::Folder {
            return Ok(BTreeSet::new());
        }
        let reloaded = load_folder(&self.root, cancellation)?;
        let previous = self
            .projects
            .iter()
            .map(|project| (project.manifest_path().to_path_buf(), project))
            .collect::<BTreeMap<_, _>>();
        let current = reloaded
            .projects
            .iter()
            .map(|project| (project.manifest_path().to_path_buf(), project))
            .collect::<BTreeMap<_, _>>();
        let mut changed = BTreeSet::new();
        for manifest in previous.keys().chain(current.keys()) {
            if previous.get(manifest) != current.get(manifest) {
                changed.insert(manifest.clone());
            }
        }
        *self = reloaded;
        Ok(changed)
    }

    pub(super) fn load_project_manifest(path: &Path) -> Self {
        match load_editor_project(path) {
            Ok(project) => Self {
                root: path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| path.to_path_buf()),
                manifest_path: Some(path.to_path_buf()),
                kind: WorkspaceKind::Project,
                projects: vec![ProjectContext::new(path, project)],
                issues: Vec::new(),
            },
            Err(message) => Self::unavailable(path, message),
        }
    }

    pub(super) fn load_workspace_manifest(path: &Path) -> Self {
        let workspace = match load_workspace(path) {
            Ok(workspace) => workspace,
            Err(message) => return Self::unavailable(path, message),
        };
        let mut projects = Vec::new();
        let mut issues = Vec::new();
        for member in workspace.member_projects {
            match load_editor_project(&member) {
                Ok(project) => projects.push(ProjectContext::new(&member, project)),
                Err(message) => issues.push(WorkspaceIssue {
                    path: normalized_path(&member),
                    message,
                }),
            }
        }
        Self {
            root: path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf()),
            manifest_path: Some(path.to_path_buf()),
            kind: WorkspaceKind::Workspace,
            projects,
            issues,
        }
    }

    pub(super) fn unavailable(path: &Path, message: String) -> Self {
        Self {
            root: path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf()),
            manifest_path: Some(path.to_path_buf()),
            kind: WorkspaceKind::Unavailable,
            projects: Vec::new(),
            issues: vec![WorkspaceIssue {
                path: path.to_path_buf(),
                message,
            }],
        }
    }

    pub(super) fn merge_discovered(&mut self, context: Self) {
        let mut added = false;
        for project in context.projects {
            if self
                .projects
                .iter()
                .any(|loaded| loaded.manifest_path() == project.manifest_path())
            {
                continue;
            }
            self.projects.push(project);
            added = true;
        }
        for issue in context.issues {
            if !self.issues.contains(&issue) {
                self.issues.push(issue);
            }
        }
        if added && self.manifest_path.is_none() {
            self.kind = WorkspaceKind::Folder;
        }
    }
}

fn ambiguous_source_issue(source: &Path, manifests: &[&Path]) -> WorkspaceIssue {
    let mut names = manifests
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    WorkspaceIssue {
        path: normalized_path(source),
        message: format!(
            "Source belongs directly to multiple FPAS projects: {}.\n  help: Adjust `[sources]` so exactly one nearest project owns this file.",
            names.join(", ")
        ),
    }
}

fn load_editor_project(path: &Path) -> Result<LoadedProject, String> {
    let is_standard_library = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("stdlib.fpasprj"));
    if is_standard_library {
        let root = path.parent().unwrap_or(path);
        load_standard_library_project(root)
    } else {
        load_project(path)
    }
}

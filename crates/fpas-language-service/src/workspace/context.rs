//! Manifest-backed workspace context.

use std::path::{Path, PathBuf};

use fpas_project::{
    LoadedProject, ProjectKind, discover_workspace_file, load_project, load_workspace,
};

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

/// One loaded project and its resolved source ownership.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    manifest_path: PathBuf,
    project: LoadedProject,
}

impl ProjectContext {
    /// Returns the normalized `.fpasprj` path.
    #[must_use]
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    /// Returns the declared project name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.project.name
    }

    /// Returns the declared project kind.
    #[must_use]
    pub fn kind(&self) -> ProjectKind {
        self.project.kind
    }

    /// Returns the optional program entry source.
    #[must_use]
    pub fn main(&self) -> Option<&Path> {
        self.project.main.as_deref()
    }

    /// Returns the complete resolved user-unit and dependency source list.
    #[must_use]
    pub fn source_files(&self) -> &[PathBuf] {
        &self.project.source_files
    }

    /// Returns whether this project owns or consumes the given source path.
    #[must_use]
    pub fn contains_source(&self, path: &Path) -> bool {
        let path = normalized_path(path);
        self.main()
            .is_some_and(|main| normalized_path(main) == path)
            || self
                .source_files()
                .iter()
                .any(|source| normalized_path(source) == path)
    }

    pub(crate) fn loaded(&self) -> &LoadedProject {
        &self.project
    }

    pub(crate) fn all_source_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.project.source_files.clone();
        if let Some(main) = &self.project.main {
            paths.push(main.clone());
        }
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Loaded editor context with recoverable manifest problems.
#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    root: PathBuf,
    manifest_path: Option<PathBuf>,
    kind: WorkspaceKind,
    projects: Vec<ProjectContext>,
    issues: Vec<WorkspaceIssue>,
}

impl WorkspaceContext {
    /// Loads an explicit source, directory, project, or workspace path.
    ///
    /// Invalid or absent metadata becomes [`WorkspaceIssue`] state instead of a constructor error.
    #[must_use]
    pub fn load(input: &Path) -> Self {
        let input = normalized_path(input);
        if has_extension(&input, "fpasprj") {
            Self::load_project_manifest(&input)
        } else if has_extension(&input, "fpasworkspace") {
            Self::load_workspace_manifest(&input)
        } else {
            Self::discover_or_loose(&input)
        }
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

    /// Finds the loaded project that contains a source, preferring a main-file ownership match.
    #[must_use]
    pub fn project_for_source(&self, path: &Path) -> Option<&ProjectContext> {
        let path = normalized_path(path);
        self.projects
            .iter()
            .find(|project| {
                project
                    .main()
                    .is_some_and(|main| normalized_path(main) == path)
            })
            .or_else(|| {
                self.projects
                    .iter()
                    .find(|project| project.contains_source(&path))
            })
    }

    fn discover_or_loose(input: &Path) -> Self {
        let start = if input.is_dir() {
            input.to_path_buf()
        } else {
            input
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| input.to_path_buf())
        };
        match discover_manifest(&start) {
            Ok(Some(path)) if has_extension(&path, "fpasworkspace") => {
                Self::load_workspace_manifest(&path)
            }
            Ok(Some(path)) => Self::load_project_manifest(&path),
            Ok(None) => Self::loose(input),
            Err(issue) => Self {
                root: input.to_path_buf(),
                manifest_path: None,
                kind: WorkspaceKind::Unavailable,
                projects: Vec::new(),
                issues: vec![issue],
            },
        }
    }

    fn load_project_manifest(path: &Path) -> Self {
        match load_project(path) {
            Ok(project) => Self {
                root: path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| path.to_path_buf()),
                manifest_path: Some(path.to_path_buf()),
                kind: WorkspaceKind::Project,
                projects: vec![ProjectContext {
                    manifest_path: path.to_path_buf(),
                    project,
                }],
                issues: Vec::new(),
            },
            Err(message) => Self::unavailable(path, message),
        }
    }

    fn load_workspace_manifest(path: &Path) -> Self {
        let workspace = match load_workspace(path) {
            Ok(workspace) => workspace,
            Err(message) => return Self::unavailable(path, message),
        };
        let mut projects = Vec::new();
        let mut issues = Vec::new();
        for member in workspace.member_projects {
            match load_project(&member) {
                Ok(project) => projects.push(ProjectContext {
                    manifest_path: normalized_path(&member),
                    project,
                }),
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

    fn unavailable(path: &Path, message: String) -> Self {
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
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
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
                        "Found multiple `.fpasprj` files while discovering editor context: {names}.\n  help: Open the desired project or workspace manifest explicitly."
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

//! Loads `.fpasworkspace` manifests (`docs/pascal/program-structure/workspaces.md`).

use crate::manifest::{invalid_value, parse_manifest, read_manifest};
use crate::paths::{
    resolve_explicit_file_path, unresolvable_root_error, validate_project_file_extension,
};
use crate::source::{validate_non_empty, validate_non_empty_entry};
use crate::{ProjectError, ProjectKind};
use fpas_diagnostics::codes::{
    PROJECT_DIRECTORY_READ_FAILED, PROJECT_DISCOVERY_FAILED, PROJECT_DUPLICATE_ENTRY,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";

/// Resolved workspace with validated member project paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedWorkspace {
    /// Declared `workspace.name`.
    pub name: String,
    /// Absolute or normalized paths to member `.fpasprj` files.
    pub member_projects: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceFile {
    workspace: WorkspaceSection,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    name: String,
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectNameFile {
    project: ProjectNameSection,
}

#[derive(Debug, Deserialize)]
struct ProjectNameSection {
    name: String,
    kind: String,
}

/// Lightweight member manifest fields used for workspace discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemberProjectManifest {
    /// Declared `project.name`.
    pub name: String,
    /// Declared `project.kind` (`program`, `library`, or `test`).
    pub kind: crate::ProjectKind,
}

/// Reads `project.name` and `project.kind` from a member `.fpasprj` without loading sources.
pub(super) fn read_member_project_manifest(
    path: &Path,
) -> Result<MemberProjectManifest, ProjectError> {
    let manifest = read_member_project_manifest_raw(path)?;
    Ok(MemberProjectManifest {
        name: manifest.name,
        kind: ProjectKind::parse(&manifest.kind, path)?,
    })
}

/// Reads `project.name` from a member `.fpasprj` without loading sources or dependencies.
pub(super) fn read_member_project_name(path: &Path) -> Result<String, ProjectError> {
    Ok(read_member_project_manifest(path)?.name)
}

fn read_member_project_manifest_raw(path: &Path) -> Result<ProjectNameSection, ProjectError> {
    let project_text = read_manifest(path, "project")?;
    let project_file: ProjectNameFile = parse_manifest(
        path,
        "project",
        &project_text,
        "Use TOML syntax with a `[project]` section.",
    )?;

    validate_non_empty("project.name", &project_file.project.name)
        .map_err(|error| error.in_file(path))?;
    validate_non_empty("project.kind", &project_file.project.kind)
        .map_err(|error| error.in_file(path))?;
    Ok(project_file.project)
}

/// Load and validate a workspace file.
///
/// Documentation: `docs/pascal/program-structure/workspaces.md`
pub fn load_workspace(path: &Path) -> Result<LoadedWorkspace, ProjectError> {
    read_workspace(path).map_err(|error| error.in_file(path))
}

fn read_workspace(path: &Path) -> Result<LoadedWorkspace, ProjectError> {
    let workspace_text = read_manifest(path, "workspace")?;
    let workspace_file: WorkspaceFile = parse_manifest(
        path,
        "workspace",
        &workspace_text,
        "Use TOML syntax with `[workspace]` and `members = [...]`.",
    )?;

    validate_non_empty("workspace.name", &workspace_file.workspace.name)?;

    let root_dir = path
        .parent()
        .ok_or_else(|| unresolvable_root_error("workspace", path))?;

    if workspace_file.workspace.members.is_empty() {
        return Err(invalid_value(
            "`workspace.members` must contain at least one project path.",
            "Add one or more `.fpasprj` paths.",
        ));
    }

    let mut member_projects = Vec::new();
    let mut seen = Vec::<PathBuf>::new();

    for member in &workspace_file.workspace.members {
        validate_non_empty_entry("workspace.members", member).map_err(|error| {
            error.with_help("Remove empty entries or provide a `.fpasprj` path.")
        })?;

        let member_path = resolve_explicit_file_path("workspace.members", member, root_dir)?;
        validate_project_file_extension(&member_path, "workspace.members")
            .map_err(|error| error.with_help("List project manifest paths only."))?;
        let key = crate::paths::canonical_project_path(&member_path);
        if seen
            .iter()
            .any(|existing| crate::paths::same_file(existing, &key))
        {
            return Err(ProjectError::new(
                PROJECT_DUPLICATE_ENTRY,
                format!(
                    "Duplicate workspace member `{}` resolves to the same project as an earlier entry.",
                    member_path.to_string_lossy()
                ),
            )
            .with_help("List each `.fpasprj` path at most once in `workspace.members`."));
        }
        seen.push(key);
        member_projects.push(member_path);
    }

    Ok(LoadedWorkspace {
        name: workspace_file.workspace.name,
        member_projects,
    })
}

/// Discover a single `.fpasworkspace` file in `cwd`, if present.
///
/// Documentation: `docs/pascal/program-structure/workspaces.md`
pub fn discover_workspace_file(cwd: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let directory_error = |error: std::io::Error| {
        ProjectError::new(
            PROJECT_DIRECTORY_READ_FAILED,
            format!(
                "Error reading current directory `{}`: {error}",
                cwd.display()
            ),
        )
    };
    let read_dir = fs::read_dir(cwd).map_err(directory_error)?;

    let mut candidates = Vec::<PathBuf>::new();
    for entry in read_dir {
        let path = entry.map_err(directory_error)?.path();
        if path.is_file() && is_workspace_file(&path) {
            candidates.push(path);
        }
    }

    candidates.sort();

    match candidates.len() {
        0 => Ok(None),
        1 => Ok(Some(candidates.remove(0))),
        _ => {
            let entries = candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(ProjectError::new(
                PROJECT_DISCOVERY_FAILED,
                format!(
                    "Found multiple `.fpasworkspace` files in current directory `{}`: {entries}.",
                    cwd.display()
                ),
            )
            .with_help("Pass the desired workspace file path explicitly."))
        }
    }
}

fn is_workspace_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(WORKSPACE_FILE_EXTENSION))
}

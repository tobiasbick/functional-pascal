//! Loads `.fpasworkspace` manifests (`docs/pascal/program-structure/workspaces.md`).

use crate::ProjectKind;
use crate::common::validate_non_empty;
use crate::paths::resolve_explicit_file_path;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

const WORKSPACE_FILE_EXTENSION: &str = "fpasworkspace";

/// Resolved workspace with validated member project paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedWorkspace {
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
pub(super) fn read_member_project_manifest(path: &Path) -> Result<MemberProjectManifest, String> {
    let manifest = read_member_project_manifest_raw(path)?;
    Ok(MemberProjectManifest {
        name: manifest.name,
        kind: ProjectKind::parse_in_file(&manifest.kind, Some(path))?,
    })
}

/// Reads `project.name` from a member `.fpasprj` without loading sources or dependencies.
pub(super) fn read_member_project_name(path: &Path) -> Result<String, String> {
    Ok(read_member_project_manifest(path)?.name)
}

fn read_member_project_manifest_raw(path: &Path) -> Result<ProjectNameSection, String> {
    let project_text = fs::read_to_string(path).map_err(|e| {
        format!(
            "Error reading project file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;

    let project_file: ProjectNameFile = toml::from_str(&project_text).map_err(|e| {
        format!(
            "Invalid project file `{}`: {e}\n  help: Use TOML syntax with a `[project]` section.",
            path.to_string_lossy()
        )
    })?;

    validate_non_empty("project.name", &project_file.project.name)?;
    validate_non_empty("project.kind", &project_file.project.kind)?;
    Ok(project_file.project)
}

/// Load and validate a workspace file.
pub fn load_workspace(path: &Path) -> Result<LoadedWorkspace, String> {
    let workspace_text = fs::read_to_string(path).map_err(|e| {
        format!(
            "Error reading workspace file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;

    let workspace_file: WorkspaceFile = toml::from_str(&workspace_text).map_err(|e| {
        format!(
            "Invalid workspace file `{}`: {e}\n  help: Use TOML syntax with `[workspace]` and `members = [...]`.",
            path.to_string_lossy()
        )
    })?;

    validate_non_empty("workspace.name", &workspace_file.workspace.name)?;

    let root_dir = path.parent().ok_or_else(|| {
        format!(
            "Cannot resolve workspace root for `{}`.\n  help: Use a normal file path inside a directory.",
            path.to_string_lossy()
        )
    })?;

    if workspace_file.workspace.members.is_empty() {
        return Err(
            "`workspace.members` must contain at least one project path.\n  help: Add one or more `.fpasprj` paths."
                .to_string(),
        );
    }

    let mut member_projects = Vec::new();
    let mut seen = Vec::<PathBuf>::new();

    for member in &workspace_file.workspace.members {
        if member.trim().is_empty() {
            return Err(
                "A `workspace.members` entry is empty.\n  help: Remove empty entries or provide a `.fpasprj` path."
                    .to_string(),
            );
        }

        let member_path = resolve_workspace_member_path(member, root_dir)?;
        let key = crate::paths::canonical_project_path(&member_path);
        if seen
            .iter()
            .any(|existing| crate::paths::same_file(existing, &key))
        {
            return Err(format!(
                "Duplicate workspace member `{}` resolves to the same project as an earlier entry.\n  help: List each `.fpasprj` path at most once in `workspace.members`.",
                member_path.to_string_lossy()
            ));
        }
        seen.push(key);
        member_projects.push(member_path);
    }

    if member_projects.is_empty() {
        return Err(
            "`workspace.members` did not resolve to any project files.\n  help: Add valid `.fpasprj` paths."
                .to_string(),
        );
    }

    Ok(LoadedWorkspace { member_projects })
}

/// Discover a single `.fpasworkspace` file in `cwd`, if present.
pub fn discover_workspace_file(cwd: &Path) -> Result<Option<PathBuf>, String> {
    let read_dir = fs::read_dir(cwd)
        .map_err(|e| format!("Error reading current directory `{}`: {e}", cwd.display()))?;

    let mut candidates = Vec::<PathBuf>::new();
    for entry in read_dir {
        let entry = entry.map_err(|e| {
            format!(
                "Error reading entries from current directory `{}`: {e}",
                cwd.display()
            )
        })?;
        let path = entry.path();
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
            Err(format!(
                "Found multiple `.fpasworkspace` files in current directory `{}`: {entries}.\n  help: Pass the desired workspace file path explicitly.",
                cwd.display()
            ))
        }
    }
}

fn resolve_workspace_member_path(member: &str, root_dir: &Path) -> Result<PathBuf, String> {
    let path = resolve_explicit_file_path("workspace.members", member, root_dir)?;
    validate_workspace_member_extension(&path)?;
    Ok(path)
}

fn validate_workspace_member_extension(path: &Path) -> Result<(), String> {
    if is_workspace_member_project(path) {
        return Ok(());
    }

    Err(format!(
        "`workspace.members` must reference a `.fpasprj` file: `{}`.\n  help: List project manifest paths only.",
        path.to_string_lossy()
    ))
}

fn is_workspace_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(WORKSPACE_FILE_EXTENSION))
}

fn is_workspace_member_project(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("fpasprj"))
}

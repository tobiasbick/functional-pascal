//! Workspace discovery helpers for the CLI run command.
//!
//! Documentation: `docs/pascal/10-projects.md`

use super::loading::{load_workspace, read_member_project_manifest};
use crate::ProjectKind;
use std::path::{Path, PathBuf};

/// Returns the sole `kind = "program"` member when a workspace has exactly one.
///
/// Documentation: `docs/pascal/10-projects.md`
pub fn discover_run_project_in_workspace(workspace_path: &Path) -> Result<PathBuf, String> {
    let workspace = load_workspace(workspace_path)?;
    let mut program_members = Vec::<PathBuf>::new();

    for member_path in &workspace.member_projects {
        let manifest = read_member_project_manifest(member_path)?;
        if manifest.kind == ProjectKind::Program {
            program_members.push(member_path.clone());
        }
    }

    match program_members.len() {
        0 => Err(
            "No `program` projects found in the workspace.\n  help: Add a `kind = \"program\"` member to `workspace.members`, or pass a `.fpasprj` path explicitly."
                .to_string(),
        ),
        1 => Ok(program_members.remove(0)),
        _ => {
            let entries = program_members
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "Found multiple `program` projects in the workspace: {entries}.\n  help: Pass the desired `.fpasprj` file path explicitly."
            ))
        }
    }
}

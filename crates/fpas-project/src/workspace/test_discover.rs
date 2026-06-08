//! Workspace discovery helpers for `fpas test`.
//!
//! Documentation: `docs/pascal/10-projects.md`, `docs/future/test-framework/runner.md`.

use super::loading::{load_workspace, read_member_project_manifest};
use crate::ProjectKind;
use std::path::{Path, PathBuf};

/// Returns all `kind = "test"` member projects in a workspace.
pub fn discover_test_projects_in_workspace(workspace_path: &Path) -> Result<Vec<PathBuf>, String> {
    let workspace = load_workspace(workspace_path)?;
    let mut test_members = Vec::new();

    for member_path in &workspace.member_projects {
        let manifest = read_member_project_manifest(member_path)?;
        if manifest.kind == ProjectKind::Test {
            test_members.push(member_path.clone());
        }
    }

    Ok(test_members)
}

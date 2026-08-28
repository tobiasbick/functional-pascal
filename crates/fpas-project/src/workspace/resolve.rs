//! Resolves `[dependencies].workspace` names against an enclosing `.fpasworkspace`.
//!
//! Documentation: `docs/pascal/program-structure/workspaces.md`

use super::loading::discover_workspace_file;
use super::loading::{load_workspace, read_member_project_name};
use crate::paths::absolute_project_path;
use caseless::default_case_fold_str;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Finds the nearest `.fpasworkspace` file starting at `start` and walking upward.
pub fn find_enclosing_workspace(start: &Path) -> Result<Option<PathBuf>, String> {
    let mut dir = start.to_path_buf();
    loop {
        if let Some(workspace_path) = discover_workspace_file(&dir)? {
            return Ok(Some(workspace_path));
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

/// Resolves workspace dependency names to member `.fpasprj` paths.
pub fn resolve_workspace_dependency_paths(
    consumer_project: &Path,
    names: &[String],
) -> Result<Vec<PathBuf>, String> {
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let consumer_project = absolute_project_path(consumer_project)?;
    let consumer_root = consumer_project.parent().ok_or_else(|| {
        format!(
            "Cannot resolve project root for `{}`.\n  help: Use a normal file path inside a directory.",
            consumer_project.to_string_lossy()
        )
    })?;

    let Some(workspace_path) = find_enclosing_workspace(consumer_root)? else {
        return Err(
            "`dependencies.workspace` requires an enclosing `.fpasworkspace` file.\n  help: Place the consumer project inside a workspace tree, or use `dependencies.projects` with a path."
                .to_string(),
        );
    };

    let workspace = load_workspace(&workspace_path)?;
    let name_index = build_workspace_project_name_index(&workspace.member_projects)?;

    let mut resolved = Vec::new();
    for name in names {
        if name.trim().is_empty() {
            return Err(
                "A `dependencies.workspace` entry is empty.\n  help: Remove empty entries or provide a member `project.name`."
                    .to_string(),
            );
        }

        let key = canonical_project_name(name.trim());
        let Some(path) = name_index.get(&key) else {
            let available = sorted_workspace_names(&name_index);
            return Err(format!(
                "Unknown workspace dependency `{name}`.\n  help: Use a `project.name` from a workspace member. Available: {available}."
            ));
        };
        resolved.push(path.clone());
    }

    Ok(resolved)
}

fn build_workspace_project_name_index(
    member_projects: &[PathBuf],
) -> Result<HashMap<String, PathBuf>, String> {
    let mut index = HashMap::<String, PathBuf>::new();

    for member_path in member_projects {
        let project_name = read_member_project_name(member_path)?;
        let key = canonical_project_name(&project_name);
        if let Some(first_path) = index.get(&key) {
            return Err(format!(
                "Duplicate workspace project name `{project_name}` in `{}` and `{}`.\n  help: Use unique `project.name` values across workspace members.",
                first_path.to_string_lossy(),
                member_path.to_string_lossy()
            ));
        }
        index.insert(key, member_path.clone());
    }

    Ok(index)
}

fn canonical_project_name(name: &str) -> String {
    default_case_fold_str(name)
}

fn sorted_workspace_names(index: &HashMap<String, PathBuf>) -> String {
    let mut names = index
        .values()
        .filter_map(|path| read_member_project_name(path).ok())
        .collect::<Vec<_>>();
    names.sort();
    names.join(", ")
}

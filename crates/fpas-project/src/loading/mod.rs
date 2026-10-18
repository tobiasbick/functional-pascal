//! Project file loading (`docs/pascal/program-structure/projects.md`).

pub(super) mod exports;
pub(super) mod own;
pub(super) mod parse_cache;

#[cfg(test)]
mod tests;

use crate::dependencies::load_project_with_dependencies;
use crate::loading::parse_cache::ParsedSourceCache;
use crate::model::LoadedProject;
use crate::paths::absolute_project_path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load and validate a Functional Pascal project file, including library dependencies.
///
/// This implements project-file handling from `docs/pascal/program-structure/projects.md`
/// and validates user-unit naming rules from `docs/pascal/program-structure/units.md`.
pub fn load_project(path: &Path) -> Result<LoadedProject, crate::ProjectError> {
    load_project_with_source_overlays(path, &HashMap::new())
}

/// Resolves a manifest's own `project.main` and included, non-excluded source paths.
///
/// Does not parse source text, load dependency projects, or validate source declarations.
/// Tools can therefore identify owned files even when those files contain syntax errors.
/// **Documentation:** `docs/pascal/program-structure/cli.md` (`fpas check --fix`).
///
/// # Errors
/// Returns manifest or source-path selection failures with their original diagnostics.
pub fn load_project_source_paths(path: &Path) -> Result<Vec<PathBuf>, crate::ProjectError> {
    let path = absolute_project_path(path)?;
    own::load_own_source_paths(&path).map_err(|error| error.in_file(&path))
}

/// Loads a manifest and dependencies using current editor sources before disk text.
///
/// Source selection and manifests remain disk-backed; supplied source paths override
/// parsing and validation throughout the dependency tree.
/// **Documentation:** `docs/pascal/tools/editor-integration.md`
pub fn load_project_with_source_overlays(
    path: &Path,
    overlays: &HashMap<PathBuf, String>,
) -> Result<LoadedProject, crate::ProjectError> {
    let path = absolute_project_path(path)?;
    let mut visiting = Vec::new();
    let mut project_cache = HashMap::new();
    let mut parse_cache = ParsedSourceCache::with_source_overlays(overlays);
    load_project_with_dependencies(&path, &mut visiting, &mut project_cache, &mut parse_cache)
}

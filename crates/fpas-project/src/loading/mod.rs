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
use std::path::Path;

/// Load and validate a Functional Pascal project file, including library dependencies.
///
/// This implements project-file handling from `docs/pascal/program-structure/projects.md`
/// and validates user-unit naming rules from `docs/pascal/program-structure/units.md`.
pub fn load_project(path: &Path) -> Result<LoadedProject, String> {
    let path = absolute_project_path(path)?;
    let mut visiting = Vec::new();
    let mut project_cache = HashMap::new();
    let mut parse_cache = ParsedSourceCache::new();
    load_project_with_dependencies(&path, &mut visiting, &mut project_cache, &mut parse_cache)
}

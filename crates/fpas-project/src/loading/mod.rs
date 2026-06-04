//! Project file loading (`docs/pascal/10-projects.md`).

pub(super) mod exports;
pub(super) mod own;

use crate::dependencies::load_project_with_dependencies;
use crate::model::LoadedProject;
use std::collections::HashMap;
use std::path::Path;

/// Load and validate a Functional Pascal project file, including library dependencies.
///
/// This implements project-file handling from `docs/pascal/10-projects.md`
/// and validates user-unit naming rules from `docs/pascal/09-units.md`.
pub fn load_project(path: &Path) -> Result<LoadedProject, String> {
    let mut visiting = Vec::new();
    let mut cache = HashMap::new();
    load_project_with_dependencies(path, &mut visiting, &mut cache)
}

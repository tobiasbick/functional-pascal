//! Resolves `[dependencies].projects` and merges library sources into a consumer.
//!
//! Documentation: `docs/pascal/10-projects.md`

use super::loading::own::{load_own_project, validate_project_source_units};
use super::model::{LoadedProject, ProjectKind};
use super::paths::{
    canonical_project_path, merge_source_files, resolve_project_dependency_path, same_file,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Loads a project together with transitive library dependencies.
///
/// Library dependency paths are resolved relative to the consuming project's root
/// or as absolute paths. Cycles and non-library dependencies are rejected.
pub(super) fn load_project_with_dependencies(
    path: &Path,
    visiting: &mut Vec<PathBuf>,
    cache: &mut HashMap<PathBuf, LoadedProject>,
) -> Result<LoadedProject, String> {
    let canonical = canonical_project_path(path);
    if let Some(cached) = cache.get(&canonical) {
        return Ok(cached.clone());
    }

    if visiting
        .iter()
        .any(|visited| same_file(visited, &canonical))
    {
        return Err(cyclic_project_dependency_error(visiting, path));
    }

    visiting.push(canonical.clone());
    let own = load_own_project(path)?;
    let mut source_files = Vec::new();
    let mut warnings = own.warnings;

    for dependency_path in dedupe_dependency_paths(&own.dependency_projects, &own.root_dir)? {
        let dependency_loaded = load_project_with_dependencies(&dependency_path, visiting, cache)?;
        ensure_library_dependency(&dependency_path, &dependency_loaded)?;
        merge_source_files(
            &mut source_files,
            dependency_loaded.source_files,
            &mut warnings,
        );
    }

    merge_source_files(&mut source_files, own.source_files, &mut warnings);
    source_files = validate_project_source_units(source_files, &mut warnings)?;

    let loaded = LoadedProject {
        kind: own.kind,
        main: own.main,
        source_files,
        warnings,
    };

    visiting.pop();
    cache.insert(canonical, loaded.clone());
    Ok(loaded)
}

fn dedupe_dependency_paths(raw_paths: &[String], root_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut resolved = Vec::new();
    let mut seen = Vec::<PathBuf>::new();

    for raw in raw_paths {
        let path = resolve_project_dependency_path(raw, root_dir)?;
        let key = canonical_project_path(&path);
        if seen.iter().any(|existing| same_file(existing, &key)) {
            continue;
        }
        seen.push(key);
        resolved.push(path);
    }

    Ok(resolved)
}

fn ensure_library_dependency(path: &Path, loaded: &LoadedProject) -> Result<(), String> {
    if loaded.kind == ProjectKind::Library {
        return Ok(());
    }

    Err(format!(
        "Project dependency `{}` must be a library project (`kind = \"library\"`).\n  help: Point `dependencies.projects` at a `.fpasprj` with `kind = \"library\"`, or change the dependency to a program-only local include.",
        path.to_string_lossy()
    ))
}

fn cyclic_project_dependency_error(visiting: &[PathBuf], path: &Path) -> String {
    let mut cycle = visiting
        .iter()
        .map(|entry| format!("`{}`", entry.to_string_lossy()))
        .collect::<Vec<_>>();
    cycle.push(format!("`{}`", path.to_string_lossy()));
    format!(
        "Cyclic project dependency detected: {}.\n  help: Remove or reorder `dependencies.projects` so library projects do not depend on each other in a cycle.",
        cycle.join(" -> ")
    )
}

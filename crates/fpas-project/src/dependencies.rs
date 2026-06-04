//! Resolves `[dependencies].projects` and merges library sources into a consumer.
//!
//! Documentation: `docs/pascal/10-projects.md`

use super::loading::own::{load_own_project, validate_project_source_units};
use super::model::{
    LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin,
};
use super::paths::{
    canonical_project_path, merge_source_files, resolve_project_dependency_path, same_file,
};
use super::workspace::resolve_workspace_dependency_paths;
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
    let mut link_meta = ProjectLinkMeta::default();

    let dependency_paths =
        resolve_all_dependency_paths(path, &own.dependency_projects, &own.workspace_dependencies)?;
    for dependency_path in dependency_paths {
        let dependency_loaded = load_project_with_dependencies(&dependency_path, visiting, cache)?;
        ensure_library_dependency(&dependency_path, &dependency_loaded)?;
        merge_dependency_link_meta(
            &mut link_meta,
            &dependency_path,
            &dependency_loaded,
        );
        merge_source_files(
            &mut source_files,
            dependency_loaded.source_files,
            &mut warnings,
        );
    }

    let own_source_paths = own.source_files.clone();
    merge_source_files(&mut source_files, own.source_files, &mut warnings);
    for source_path in &own_source_paths {
        link_meta
            .source_origins
            .insert(source_path.clone(), SourceOrigin::Own);
    }
    source_files = validate_project_source_units(source_files, &mut warnings)?;

    let export_policy_for_dependents = match own.kind {
        ProjectKind::Library => own.export_policy.clone(),
        ProjectKind::Program => LibraryExportPolicy::AllUnits,
    };

    let loaded = LoadedProject {
        kind: own.kind,
        main: own.main,
        source_files,
        warnings,
        link_meta,
        export_policy_for_dependents,
    };

    visiting.pop();
    cache.insert(canonical, loaded.clone());
    Ok(loaded)
}

fn resolve_all_dependency_paths(
    consumer_project: &Path,
    project_paths: &[String],
    workspace_names: &[String],
) -> Result<Vec<PathBuf>, String> {
    let root_dir = consumer_project.parent().ok_or_else(|| {
        format!(
            "Cannot resolve project root for `{}`.\n  help: Use a normal file path inside a directory.",
            consumer_project.to_string_lossy()
        )
    })?;

    let mut resolved = Vec::new();
    let mut seen = Vec::<PathBuf>::new();

    for raw in project_paths {
        insert_dependency_path(
            resolve_project_dependency_path(raw, root_dir)?,
            &mut resolved,
            &mut seen,
        );
    }

    for path in resolve_workspace_dependency_paths(consumer_project, workspace_names)? {
        insert_dependency_path(path, &mut resolved, &mut seen);
    }

    Ok(resolved)
}

fn insert_dependency_path(path: PathBuf, resolved: &mut Vec<PathBuf>, seen: &mut Vec<PathBuf>) {
    let key = canonical_project_path(&path);
    if seen.iter().any(|existing| same_file(existing, &key)) {
        return;
    }
    seen.push(key);
    resolved.push(path);
}

/// Merges a dependency's link metadata into the consumer, preserving transitive library origins.
fn merge_dependency_link_meta(
    consumer: &mut ProjectLinkMeta,
    dependency_path: &Path,
    dependency_loaded: &LoadedProject,
) {
    let dependency_canonical = canonical_project_path(dependency_path);
    consumer.library_export_policies.insert(
        dependency_canonical.clone(),
        dependency_loaded.export_policy_for_dependents(),
    );
    consumer
        .library_export_policies
        .extend(dependency_loaded.link_meta.library_export_policies.clone());

    for source_path in &dependency_loaded.source_files {
        let origin = dependency_loaded
            .link_meta
            .source_origins
            .get(source_path)
            .cloned()
            .unwrap_or(SourceOrigin::Own);
        let remapped = match origin {
            SourceOrigin::Own => SourceOrigin::Library(dependency_canonical.clone()),
            SourceOrigin::Library(path) => SourceOrigin::Library(path),
        };
        consumer.source_origins.insert(source_path.clone(), remapped);
    }
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

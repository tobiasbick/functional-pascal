//! Resolves `[dependencies].projects` and merges library sources into a consumer.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`

use super::loading::own::{load_own_project, validate_project_source_units};
use super::loading::parse_cache::ParsedSourceCache;
use super::model::{
    LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin,
};
use super::paths::{
    canonical_project_path, canonical_source_path, merge_source_files,
    resolve_project_dependency_path, same_file,
};
use super::test_sources::validate_project_test_sources;
use super::workspace::resolve_workspace_dependency_paths;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Loads a project together with transitive library dependencies.
///
/// Library dependency paths are resolved relative to the consuming project's root
/// or as absolute paths. Cycles, non-library dependencies, and source ownership
/// conflicts between different projects are rejected.
pub(super) fn load_project_with_dependencies(
    path: &Path,
    visiting: &mut Vec<PathBuf>,
    cache: &mut HashMap<PathBuf, LoadedProject>,
    parse_cache: &mut ParsedSourceCache,
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
    let own = load_own_project(path, parse_cache)?;
    let mut source_files = Vec::new();
    let mut warnings = own.warnings;
    let mut link_meta = ProjectLinkMeta::default();

    let dependency_paths =
        resolve_all_dependency_paths(path, &own.dependency_projects, &own.workspace_dependencies)?;
    for dependency_path in dependency_paths {
        let dependency_loaded =
            load_project_with_dependencies(&dependency_path, visiting, cache, parse_cache)?;
        ensure_library_dependency(&dependency_path, &dependency_loaded)?;
        merge_dependency_link_meta(&mut link_meta, &dependency_path, &dependency_loaded)?;
        merge_source_files(
            &mut source_files,
            dependency_loaded.source_files,
            &mut warnings,
        );
    }

    reject_own_source_overlap(path, &own.source_files, &link_meta)?;
    let own_source_paths = own.source_files.clone();
    merge_source_files(&mut source_files, own.source_files, &mut warnings);
    source_files = match own.kind {
        ProjectKind::Test => {
            validate_project_test_sources(source_files, &mut warnings, parse_cache)?
        }
        ProjectKind::Program | ProjectKind::Library => {
            validate_project_source_units(source_files, &mut warnings, parse_cache)?
        }
    };
    prune_link_meta_origins(&mut link_meta, &source_files);
    mark_own_source_origins(&mut link_meta, &source_files, &own_source_paths);

    let export_policy_for_dependents = match own.kind {
        ProjectKind::Library => own.export_policy.clone(),
        ProjectKind::Program | ProjectKind::Test => LibraryExportPolicy::AllUnits,
    };

    let loaded = LoadedProject {
        name: own.name,
        kind: own.kind,
        main: own.main,
        source_files,
        warnings,
        link_meta,
        export_policy_for_dependents,
        test_manifest: own.test_manifest,
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
    let mut seen = HashSet::<PathBuf>::new();

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

fn insert_dependency_path(path: PathBuf, resolved: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>) {
    let key = canonical_project_path(&path);
    if !seen.insert(key) {
        return;
    }
    resolved.push(path);
}

/// Merges a dependency's link metadata into the consumer, preserving transitive library origins.
fn merge_dependency_link_meta(
    consumer: &mut ProjectLinkMeta,
    dependency_path: &Path,
    dependency_loaded: &LoadedProject,
) -> Result<(), String> {
    let dependency_canonical = canonical_project_path(dependency_path);
    consumer.library_export_policies.insert(
        dependency_canonical.clone(),
        dependency_loaded.export_policy_for_dependents.clone(),
    );
    consumer
        .library_export_policies
        .extend(dependency_loaded.link_meta.library_export_policies.clone());
    consumer.trusted_standard_library_sources.extend(
        dependency_loaded
            .link_meta
            .trusted_standard_library_sources
            .iter()
            .cloned(),
    );

    for source_path in &dependency_loaded.source_files {
        let origin = dependency_loaded.link_meta.origin_for_source(source_path);
        let remapped = match origin {
            SourceOrigin::Own => SourceOrigin::Library(dependency_canonical.clone()),
            SourceOrigin::Library(path) => SourceOrigin::Library(path),
        };
        reject_library_source_overlap(consumer, source_path, &remapped)?;
        consumer
            .source_origins
            .insert(canonical_source_path(source_path), remapped);
    }

    Ok(())
}

fn reject_library_source_overlap(
    link_meta: &ProjectLinkMeta,
    source_path: &Path,
    incoming_origin: &SourceOrigin,
) -> Result<(), String> {
    let SourceOrigin::Library(incoming_owner) = incoming_origin else {
        return Ok(());
    };
    let Some(existing_owner) = library_owner_for_source(link_meta, source_path) else {
        return Ok(());
    };
    if same_file(existing_owner, incoming_owner) {
        return Ok(());
    }

    Err(source_ownership_conflict_error(
        source_path,
        existing_owner,
        incoming_owner,
    ))
}

fn reject_own_source_overlap(
    project_path: &Path,
    own_source_paths: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<(), String> {
    for source_path in own_source_paths {
        let Some(library_owner) = library_owner_for_source(link_meta, source_path) else {
            continue;
        };
        return Err(source_ownership_conflict_error(
            source_path,
            library_owner,
            project_path,
        ));
    }

    Ok(())
}

fn library_owner_for_source<'a>(
    link_meta: &'a ProjectLinkMeta,
    source_path: &Path,
) -> Option<&'a Path> {
    link_meta
        .source_origins
        .iter()
        .find_map(|(recorded_path, origin)| {
            if !same_file(recorded_path, source_path) {
                return None;
            }
            match origin {
                SourceOrigin::Library(project_path) => Some(project_path.as_path()),
                SourceOrigin::Own => None,
            }
        })
}

fn source_ownership_conflict_error(
    source_path: &Path,
    first_project: &Path,
    conflicting_project: &Path,
) -> String {
    format!(
        "Source file `{}` is owned by more than one project.\n  first project: `{}`\n  conflicting project: `{}`\n  help: Keep the file in one project's `[sources]`; consume it from other projects through that library dependency.",
        canonical_source_path(source_path).to_string_lossy(),
        first_project.to_string_lossy(),
        conflicting_project.to_string_lossy()
    )
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

fn prune_link_meta_origins(link_meta: &mut ProjectLinkMeta, source_files: &[PathBuf]) {
    link_meta
        .source_origins
        .retain(|path, _| source_files.iter().any(|source| same_file(source, path)));
    link_meta
        .trusted_standard_library_sources
        .retain(|path| source_files.iter().any(|source| same_file(source, path)));
}

fn mark_own_source_origins(
    link_meta: &mut ProjectLinkMeta,
    source_files: &[PathBuf],
    own_source_paths: &[PathBuf],
) {
    for source_path in source_files {
        if own_source_paths
            .iter()
            .any(|own_path| same_file(own_path, source_path))
        {
            link_meta
                .source_origins
                .insert(canonical_source_path(source_path), SourceOrigin::Own);
        }
    }
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

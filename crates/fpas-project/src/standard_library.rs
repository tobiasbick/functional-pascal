//! Loading of the trusted, manifest-backed source standard library.
//!
//! Documentation: `docs/pascal/program-structure/projects.md` and
//! `docs/pascal/std/README.md`.

use crate::loading::own::{load_own_project, validate_standard_library_source_units};
use crate::loading::parse_cache::ParsedSourceCache;
use crate::paths::{canonical_project_path, canonical_source_path};
use crate::{LoadedProject, ProjectError, ProjectKind, ProjectLinkMeta, SourceOrigin};
use fpas_diagnostics::codes::PROJECT_STANDARD_LIBRARY_INVALID;
use fpas_std::STD_UNITS_INTRINSIC;
use std::path::{Path, PathBuf};

const STANDARD_LIBRARY_MANIFEST: &str = "stdlib.fpasprj";

/// Source files loaded from the implementation-owned standard-library root.
#[derive(Debug, Clone)]
pub struct StandardLibrary {
    source_files: Vec<PathBuf>,
    link_meta: ProjectLinkMeta,
}

impl StandardLibrary {
    /// Returns the standard-library unit source files in stable path order.
    pub fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    /// Returns the export rules and origins for the trusted source units.
    pub fn link_meta(&self) -> &ProjectLinkMeta {
        &self.link_meta
    }
}

/// Loads the standard-library manifest below an implementation-owned library root.
pub fn load_standard_library(root: &Path) -> Result<StandardLibrary, crate::ProjectError> {
    let (manifest, own, source_files) = load_standard_library_sources(root)?;
    let mut link_meta = ProjectLinkMeta::default();
    link_meta
        .library_export_policies
        .insert(canonical_project_path(&manifest), own.export_policy);
    for source_file in &source_files {
        link_meta.source_origins.insert(
            canonical_source_path(source_file),
            SourceOrigin::Library(canonical_project_path(&manifest)),
        );
        link_meta
            .trusted_standard_library_sources
            .insert(canonical_source_path(source_file));
    }

    Ok(StandardLibrary {
        source_files,
        link_meta,
    })
}

/// Loads the implementation-owned standard library as an editable project.
///
/// Sources retain their trusted standard-library provenance so overlay-safe
/// editor graphs accept the reserved `Std.*` namespace.
pub fn load_standard_library_project(root: &Path) -> Result<LoadedProject, crate::ProjectError> {
    let (_, own, source_files) = load_standard_library_sources(root)?;
    let mut link_meta = ProjectLinkMeta::default();
    for source_file in &source_files {
        link_meta
            .source_origins
            .insert(canonical_source_path(source_file), SourceOrigin::Own);
        link_meta
            .trusted_standard_library_sources
            .insert(canonical_source_path(source_file));
    }

    Ok(LoadedProject {
        name: own.name,
        kind: own.kind,
        main: own.main,
        source_files,
        warnings: own.warnings,
        link_meta,
        export_policy_for_dependents: own.export_policy,
        test_manifest: own.test_manifest,
    })
}

fn load_standard_library_sources(
    root: &Path,
) -> Result<(PathBuf, crate::loading::own::OwnProject, Vec<PathBuf>), crate::ProjectError> {
    if !root.is_dir() {
        return Err(invalid_standard_library(
            format!(
                "Standard library directory `{}` does not exist.",
                root.display()
            ),
            format!("Pass `--std-lib <directory>` containing `{STANDARD_LIBRARY_MANIFEST}`."),
        ));
    }

    let manifest = root.join(STANDARD_LIBRARY_MANIFEST);
    if !manifest.is_file() {
        return Err(invalid_standard_library(
            format!(
                "Standard library manifest `{}` does not exist.",
                manifest.display()
            ),
            format!(
                "Add `{STANDARD_LIBRARY_MANIFEST}` with `kind = \"library\"` and a `[sources]` section."
            ),
        ));
    }

    let mut parse_cache = ParsedSourceCache::new();
    let mut own = load_own_project(&manifest, &mut parse_cache)?;
    if own.kind != ProjectKind::Library {
        return Err(invalid_standard_library(
            format!(
                "Standard library manifest `{}` must declare `project.kind = \"library\"`.",
                manifest.display()
            ),
            "Change `[project].kind` to `\"library\"`.".to_string(),
        ));
    }
    if !own.dependency_projects.is_empty() || !own.workspace_dependencies.is_empty() {
        return Err(invalid_standard_library(
            format!(
                "Standard library manifest `{}` must list all trusted sources directly and cannot declare dependencies.",
                manifest.display()
            ),
            "Move the required `Std.*` source paths into `[sources].include`.".to_string(),
        ));
    }

    let source_files = validate_standard_library_source_units(
        std::mem::take(&mut own.source_files),
        &mut parse_cache,
    )?;
    validate_intrinsic_collisions(&source_files, &mut parse_cache)?;
    Ok((canonical_project_path(&manifest), own, source_files))
}

fn validate_intrinsic_collisions(
    source_files: &[PathBuf],
    parse_cache: &mut ParsedSourceCache,
) -> Result<(), crate::ProjectError> {
    for source_file in source_files {
        let (parsed, _) = parse_cache.parse(source_file, 0)?;
        let fpas_parser::CompilationUnit::Unit(unit) = parsed else {
            unreachable!("standard-library source validation accepts units only")
        };
        let name = crate::source::qualified_id_to_string(&unit.name);
        if STD_UNITS_INTRINSIC
            .iter()
            .any(|intrinsic| intrinsic.eq_ignore_ascii_case(&name))
        {
            return Err(invalid_standard_library(
                format!(
                    "Source standard-library unit `{name}` in `{}` collides with intrinsic unit `{name}`.",
                    source_file.display()
                ),
                "Choose a distinct `Std.*` unit name; source units cannot replace individual intrinsic units.".to_string(),
            ));
        }
    }
    Ok(())
}

fn invalid_standard_library(message: String, help: String) -> ProjectError {
    ProjectError::new(PROJECT_STANDARD_LIBRARY_INVALID, message).with_help(help)
}

#[cfg(test)]
mod tests;

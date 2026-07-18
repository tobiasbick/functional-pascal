//! Loading of the trusted, manifest-backed source standard library.
//!
//! Documentation: `docs/pascal/program-structure/projects.md` and
//! `docs/pascal/std/README.md`.

use crate::loading::own::{load_own_project, validate_standard_library_source_units};
use crate::loading::parse_cache::ParsedSourceCache;
use crate::{ProjectKind, ProjectLinkMeta, SourceOrigin};
use fpas_parser::{CompilationUnit, Unit};
use fpas_std::STD_UNITS_KNOWN;
use std::path::{Path, PathBuf};

const STANDARD_LIBRARY_MANIFEST: &str = "stdlib.fpasprj";

/// Source files loaded from the implementation-owned standard-library root.
#[derive(Debug, Clone)]
pub struct StandardLibrary {
    source_files: Vec<PathBuf>,
    parsed_units: Vec<ParsedStandardLibraryUnit>,
    link_meta: ProjectLinkMeta,
}

#[derive(Debug, Clone)]
struct ParsedStandardLibraryUnit {
    path: PathBuf,
    unit: Unit,
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

    /// Returns the validated parsed units cached for in-memory linking.
    pub(crate) fn parsed_units(&self) -> impl Iterator<Item = (&Path, &Unit)> {
        self.parsed_units
            .iter()
            .map(|parsed| (parsed.path.as_path(), &parsed.unit))
    }
}

/// Loads the standard-library manifest below an implementation-owned library root.
pub fn load_standard_library(root: &Path) -> Result<StandardLibrary, String> {
    if !root.is_dir() {
        return Err(format!(
            "Standard library directory `{}` does not exist.\n  help: Pass `--std-lib <directory>` containing `{STANDARD_LIBRARY_MANIFEST}`.",
            root.display()
        ));
    }

    let manifest = root.join(STANDARD_LIBRARY_MANIFEST);
    if !manifest.is_file() {
        return Err(format!(
            "Standard library manifest `{}` does not exist.\n  help: Add `{STANDARD_LIBRARY_MANIFEST}` with `kind = \"library\"` and a `[sources]` section.",
            manifest.display()
        ));
    }

    let mut parse_cache = ParsedSourceCache::new();
    let own = load_own_project(&manifest, &mut parse_cache)?;
    if own.kind != ProjectKind::Library {
        return Err(format!(
            "Standard library manifest `{}` must declare `project.kind = \"library\"`.\n  help: Change `[project].kind` to `\"library\"`.",
            manifest.display()
        ));
    }
    if !own.dependency_projects.is_empty() || !own.workspace_dependencies.is_empty() {
        return Err(format!(
            "Standard library manifest `{}` must list all trusted sources directly and cannot declare dependencies.\n  help: Move the required `Std.*` source paths into `[sources].include`.",
            manifest.display()
        ));
    }

    let source_files = validate_standard_library_source_units(own.source_files, &mut parse_cache)?;
    let parsed_units = load_parsed_units(&source_files, &mut parse_cache)?;

    let canonical_manifest = crate::paths::canonical_project_path(&manifest);
    let mut link_meta = ProjectLinkMeta::default();
    link_meta
        .library_export_policies
        .insert(canonical_manifest.clone(), own.export_policy);
    for source_file in &source_files {
        link_meta.source_origins.insert(
            source_file.clone(),
            SourceOrigin::Library(canonical_manifest.clone()),
        );
    }

    Ok(StandardLibrary {
        source_files,
        parsed_units,
        link_meta,
    })
}

fn load_parsed_units(
    source_files: &[PathBuf],
    parse_cache: &mut ParsedSourceCache,
) -> Result<Vec<ParsedStandardLibraryUnit>, String> {
    let mut parsed_units = Vec::with_capacity(source_files.len());
    for source_file in source_files {
        let (unit, _) = parse_cache.parse(source_file, 0)?;
        let CompilationUnit::Unit(unit) = unit else {
            unreachable!("standard-library source validation accepts units only");
        };
        let name = crate::common::qualified_id_to_string(&unit.name);
        if STD_UNITS_KNOWN
            .iter()
            .any(|intrinsic| intrinsic.eq_ignore_ascii_case(&name))
        {
            return Err(format!(
                "Source standard-library unit `{name}` in `{}` collides with intrinsic unit `{name}`.\n  help: Choose a distinct `Std.*` unit name; source units cannot replace individual intrinsic units.",
                source_file.display()
            ));
        }
        parsed_units.push(ParsedStandardLibraryUnit {
            path: source_file.clone(),
            unit,
        });
    }
    Ok(parsed_units)
}

#[cfg(test)]
mod tests;

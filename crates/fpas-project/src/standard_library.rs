//! Loading of the trusted, manifest-backed source standard library.
//!
//! Documentation: `docs/pascal/program-structure/projects.md` and
//! `docs/pascal/std/README.md`.

use crate::loading::own::{load_own_project, validate_standard_library_source_units};
use crate::loading::parse_cache::ParsedSourceCache;
use crate::{ProjectKind, ProjectLinkMeta, SourceOrigin};
use fpas_std::STD_UNITS_KNOWN;
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
    validate_intrinsic_collisions(&source_files)?;

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
        link_meta,
    })
}

fn validate_intrinsic_collisions(source_files: &[PathBuf]) -> Result<(), String> {
    for source_file in source_files {
        let (header, _) = crate::common::read_source_header(source_file, 0)?;
        let crate::common::SourceHeader::Unit(unit) = header else {
            unreachable!("standard-library source validation accepts units only")
        };
        let name = crate::common::qualified_id_to_string(&unit);
        if !name.eq_ignore_ascii_case("Std.Tui")
            && STD_UNITS_KNOWN
                .iter()
                .any(|intrinsic| intrinsic.eq_ignore_ascii_case(&name))
        {
            return Err(format!(
                "Source standard-library unit `{name}` in `{}` collides with intrinsic unit `{name}`.\n  help: Choose a distinct `Std.*` unit name; source units cannot replace individual intrinsic units.",
                source_file.display()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

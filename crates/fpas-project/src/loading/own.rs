//! Loads a single `.fpasprj` manifest without merging dependency projects.
//!
//! Spec: `docs/pascal/program-structure/projects.md`

use super::exports::validate_library_exports;
use crate::ProjectError;
use crate::loading::parse_cache::ParsedSourceCache;
use crate::manifest::{invalid_value, parse_manifest, read_manifest};
use crate::model::{LibraryExportPolicy, ProjectKind};
use crate::paths::{
    resolve_explicit_file_path, resolve_source_files, same_file, unresolvable_root_error,
    validate_source_extension,
};
use crate::source::{
    duplicate_unit_error, program_source_skipped, qualified_id_to_string, validate_non_empty,
    validate_non_empty_entry, validate_user_unit_name,
};
use crate::test_manifest::{TestManifest, TestSectionRaw, parse_test_section};
use fpas_diagnostics::FileDiagnostic;
use fpas_diagnostics::codes::{
    PROJECT_DUPLICATE_UNIT, PROJECT_UNIT_KIND_MISMATCH, PROJECT_UNIT_NAMESPACE_INVALID,
};
use fpas_parser::CompilationUnit;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One project's own metadata before dependency merging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnProject {
    /// Declared `project.name`.
    pub name: String,
    /// Declared project kind.
    pub kind: ProjectKind,
    /// Main program file for executable projects.
    pub main: Option<PathBuf>,
    /// Validated user-unit source files from this project's `[sources]`.
    pub source_files: Vec<PathBuf>,
    /// Paths from `[dependencies].projects` (unresolved strings).
    pub dependency_projects: Vec<String>,
    /// Names from `[dependencies].workspace` (resolved via enclosing `.fpasworkspace`).
    pub workspace_dependencies: Vec<String>,
    /// Non-fatal loading warnings such as duplicate include entries.
    pub warnings: Vec<FileDiagnostic>,
    /// Export policy applied when this library is consumed as a dependency.
    pub export_policy: LibraryExportPolicy,
    /// Optional `[test]` overrides for `fpas test` when `kind = "test"`.
    pub test_manifest: TestManifest,
}

#[derive(Debug, Deserialize)]
struct ProjectFile {
    project: ProjectSection,
    sources: Option<SourcesSection>,
    dependencies: Option<DependenciesSection>,
    exports: Option<ExportsSection>,
    test: Option<TestSectionRaw>,
}

#[derive(Debug, Deserialize)]
struct ExportsSection {
    units: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectSection {
    name: String,
    version: Option<String>,
    kind: String,
    main: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SourcesSection {
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DependenciesSection {
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    workspace: Vec<String>,
}

/// Parse and validate one project file's own sources and metadata.
pub(crate) fn load_own_project(
    path: &Path,
    parse_cache: &mut ParsedSourceCache,
) -> Result<OwnProject, crate::ProjectError> {
    read_own_project(path, parse_cache).map_err(|error| error.in_file(path))
}

fn read_own_project(
    path: &Path,
    parse_cache: &mut ParsedSourceCache,
) -> Result<OwnProject, crate::ProjectError> {
    let project_text = read_manifest(path, "project")?;
    let project_file: ProjectFile = parse_manifest(
        path,
        "project",
        &project_text,
        "Use TOML syntax with `[project]` and `[sources]` sections.",
    )?;

    validate_non_empty("project.name", &project_file.project.name)?;
    validate_optional_non_empty("project.version", project_file.project.version.as_deref())?;

    let kind = ProjectKind::parse(&project_file.project.kind, path)?;
    let root_dir = path
        .parent()
        .ok_or_else(|| unresolvable_root_error("project", path))?;

    let sources = project_file.sources.ok_or_else(|| {
        invalid_value(
            "Missing `[sources]` section.",
            "Add `[sources]` with `include = [\"src/**/*.fpas\"]`.",
        )
    })?;

    if sources.include.is_empty() {
        return Err(invalid_value(
            "`sources.include` must contain at least one entry.",
            "Add one or more file paths or glob patterns.",
        ));
    }

    let (dependency_projects, workspace_dependencies) = project_file
        .dependencies
        .map(|section| (section.projects, section.workspace))
        .unwrap_or_default();

    validate_dependency_entries("dependencies.projects", &dependency_projects)?;
    validate_dependency_entries("dependencies.workspace", &workspace_dependencies)?;

    validate_dependency_entries("sources.exclude", &sources.exclude)?;

    let parsed_exports = parse_exports_section(kind, project_file.exports.as_ref(), path)?;

    let (mut source_files, mut warnings) =
        resolve_source_files(&sources.include, &sources.exclude, root_dir)?;
    let main = match kind {
        ProjectKind::Program => {
            let main_raw = project_file.project.main.as_deref().ok_or_else(|| {
                invalid_value(
                    "Program projects require `project.main`.",
                    "Set `main = \"src/main.fpas\"` in `[project]`.",
                )
            })?;
            let main_path = resolve_explicit_file_path("project.main", main_raw, root_dir)?;
            validate_source_extension(&main_path, "project.main")?;
            source_files.retain(|source| !same_file(source, &main_path));
            Some(main_path)
        }
        ProjectKind::Library => {
            if project_file.project.main.is_some() {
                return Err(invalid_value(
                    "Library projects must not define `project.main`.",
                    "Remove the `main` entry or change `project.kind` to `program`.",
                ));
            }
            None
        }
        ProjectKind::Test => {
            if project_file.project.main.is_some() {
                return Err(invalid_value(
                    "Test projects must not define `project.main`.",
                    "Entry files are discovered by `*_test.fpas` naming; run them with `fpas test`.",
                ));
            }
            None
        }
    };

    if let Some(main_path) = main.as_deref() {
        validate_program_main_file(main_path, &mut warnings, parse_cache)?;
    }

    let export_policy = resolve_export_policy(kind, parsed_exports, &source_files, parse_cache)?;
    let test_manifest = parse_test_section(kind, project_file.test, &source_files, root_dir, path)?;

    Ok(OwnProject {
        name: project_file.project.name,
        kind,
        main,
        source_files,
        dependency_projects,
        workspace_dependencies,
        warnings,
        export_policy,
        test_manifest,
    })
}

enum ParsedExports {
    None,
    UnitNames(Vec<String>),
}

fn parse_exports_section(
    kind: ProjectKind,
    exports: Option<&ExportsSection>,
    project_path: &Path,
) -> Result<ParsedExports, crate::ProjectError> {
    let Some(section) = exports else {
        return Ok(ParsedExports::None);
    };

    if kind != ProjectKind::Library {
        return Err(invalid_value(
            format!(
                "{} project `{}` must not define `[exports]`.",
                kind.label(),
                project_path.to_string_lossy()
            ),
            "Remove `[exports]` or change `project.kind` to `library`.",
        ));
    }

    Ok(ParsedExports::UnitNames(section.units.clone()))
}

fn resolve_export_policy(
    kind: ProjectKind,
    parsed: ParsedExports,
    source_files: &[PathBuf],
    parse_cache: &mut ParsedSourceCache,
) -> Result<LibraryExportPolicy, crate::ProjectError> {
    match (kind, parsed) {
        (_, ParsedExports::None) => Ok(LibraryExportPolicy::AllUnits),
        (ProjectKind::Library, ParsedExports::UnitNames(names)) => {
            let listed_units = validate_library_exports(&names, source_files, parse_cache)?;
            Ok(LibraryExportPolicy::ListedUnits(listed_units))
        }
        (_, ParsedExports::UnitNames(_)) => Ok(LibraryExportPolicy::AllUnits),
    }
}

fn validate_dependency_entries(
    field_name: &str,
    entries: &[String],
) -> Result<(), crate::ProjectError> {
    for entry in entries {
        validate_non_empty_entry(field_name, entry)?;
    }

    Ok(())
}

fn validate_optional_non_empty(
    field_name: &str,
    value: Option<&str>,
) -> Result<(), crate::ProjectError> {
    if let Some(value) = value {
        validate_non_empty(field_name, value)?;
    }

    Ok(())
}

fn validate_program_main_file(
    main_path: &Path,
    warnings: &mut Vec<FileDiagnostic>,
    parse_cache: &mut ParsedSourceCache,
) -> Result<(), crate::ProjectError> {
    let (unit, parse_warnings) = parse_cache.parse(main_path, 0)?;
    warnings.extend(parse_warnings);

    match unit {
        CompilationUnit::Program(_) => Ok(()),
        CompilationUnit::Unit(unit) => Err(ProjectError::new(
            PROJECT_UNIT_KIND_MISMATCH,
            format!(
                "`project.main` must declare `program`, but `{}` declares `unit {}`.",
                main_path.to_string_lossy(),
                qualified_id_to_string(&unit.name)
            ),
        )
        .with_help("Use a `program` declaration in the main file.")
        .in_file(main_path)),
    }
}

/// Validates unit declarations and rejects duplicate unit names across `source_files`.
pub(crate) fn validate_project_source_units(
    source_files: Vec<PathBuf>,
    warnings: &mut Vec<FileDiagnostic>,
    parse_cache: &mut ParsedSourceCache,
) -> Result<Vec<PathBuf>, crate::ProjectError> {
    let mut validated = Vec::new();
    let mut seen_unit_names = HashMap::<String, PathBuf>::new();

    for source_path in source_files {
        let (unit, parse_warnings) = parse_cache.parse(&source_path, 0)?;
        warnings.extend(parse_warnings);

        match unit {
            CompilationUnit::Program(program) => {
                warnings.push(program_source_skipped(
                    &source_path,
                    &program.name,
                    "Source files must use `unit` declarations.",
                ));
            }
            CompilationUnit::Unit(unit) => {
                validate_user_unit_name(&source_path, &unit.name)?;
                let unit_name = qualified_id_to_string(&unit.name);
                let key = unit_name.to_ascii_lowercase();
                if let Some(first_path) = seen_unit_names.get(&key) {
                    return Err(duplicate_unit_error(&unit_name, first_path, &source_path));
                }
                seen_unit_names.insert(key, source_path.clone());
                validated.push(source_path);
            }
        }
    }

    Ok(validated)
}

/// Validates trusted standard-library sources declared by `stdlib.fpasprj`.
pub(crate) fn validate_standard_library_source_units(
    source_files: Vec<PathBuf>,
    parse_cache: &mut ParsedSourceCache,
) -> Result<Vec<PathBuf>, crate::ProjectError> {
    let mut validated = Vec::new();
    let mut seen_unit_names = HashMap::<String, PathBuf>::new();

    for source_path in source_files {
        let (unit, _) = parse_cache.parse(&source_path, 0)?;
        let CompilationUnit::Unit(unit) = unit else {
            let CompilationUnit::Program(program) = unit else {
                unreachable!("compilation unit is program or unit");
            };
            return Err(ProjectError::new(
                PROJECT_UNIT_KIND_MISMATCH,
                format!(
                    "Standard library source file `{}` declares `program {}`.",
                    source_path.display(),
                    program.name
                ),
            )
            .with_help("Standard library manifests may include `unit Std.*` files only."));
        };

        let unit_name = qualified_id_to_string(&unit.name);
        let is_std_unit = unit.name.parts.len() >= 2
            && unit
                .name
                .parts
                .first()
                .is_some_and(|head| head.eq_ignore_ascii_case("std"));
        if !is_std_unit {
            return Err(ProjectError::new(
                PROJECT_UNIT_NAMESPACE_INVALID,
                format!(
                    "Standard library source file `{}` declares `unit {unit_name}`.",
                    source_path.display()
                ),
            )
            .with_help("Trusted standard-library units must use the `Std.*` namespace."));
        }

        let key = unit_name.to_ascii_lowercase();
        if let Some(first_path) = seen_unit_names.get(&key) {
            return Err(ProjectError::new(
                PROJECT_DUPLICATE_UNIT,
                format!(
                    "Duplicate standard-library unit name `{unit_name}` found in `{}` and `{}`.",
                    first_path.display(),
                    source_path.display()
                ),
            )
            .with_help("Use a unique `Std.*` namespace per source file."));
        }
        seen_unit_names.insert(key, source_path.clone());
        validated.push(source_path);
    }

    Ok(validated)
}

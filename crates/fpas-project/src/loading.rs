use super::common::{parse_compilation_unit_file, qualified_id_to_string, validate_user_unit_name};
use super::paths::{
    resolve_explicit_file_path, resolve_source_files, same_file, validate_source_extension,
};
use super::{LoadedProject, ProjectKind};
use fpas_lexer::DefineSet;
use fpas_parser::CompilationUnit;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ProjectFile {
    project: ProjectSection,
    sources: Option<SourcesSection>,
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
}

/// Load and validate a Functional Pascal project file.
///
/// This implements project-file handling from `docs/pascal/10-projects.md`
/// and validates user-unit naming rules from `docs/pascal/09-units.md`.
pub fn load_project(path: &Path) -> Result<LoadedProject, String> {
    load_project_with_defines(path, &DefineSet::new())
}

/// Load and validate a Functional Pascal project file with predefined
/// conditional symbols.
///
/// This implements project-file handling from `docs/pascal/10-projects.md`
/// and compiler directives from `docs/pascal/12-compiler-directives.md`.
pub fn load_project_with_defines(path: &Path, defines: &DefineSet) -> Result<LoadedProject, String> {
    let project_text = fs::read_to_string(path).map_err(|e| {
        format!(
            "Error reading project file `{}`: {e}",
            path.to_string_lossy()
        )
    })?;

    let project_file: ProjectFile = toml::from_str(&project_text).map_err(|e| {
        format!(
            "Invalid project file `{}`: {e}\n  help: Use TOML syntax with `[project]` and `[sources]` sections.",
            path.to_string_lossy()
        )
    })?;

    validate_non_empty("project.name", &project_file.project.name)?;
    validate_optional_non_empty("project.version", project_file.project.version.as_deref())?;

    let kind = parse_project_kind(&project_file.project.kind)?;
    let root_dir = path.parent().ok_or_else(|| {
        format!(
            "Cannot resolve project root for `{}`.\n  help: Use a normal file path inside a directory.",
            path.to_string_lossy()
        )
    })?;

    let sources = project_file.sources.ok_or_else(|| {
        "Missing `[sources]` section.\n  help: Add `[sources]` with `include = [\"src/**/*.fpas\"]`."
            .to_string()
    })?;

    if sources.include.is_empty() {
        return Err(
            "`sources.include` must contain at least one entry.\n  help: Add one or more file paths or glob patterns."
                .to_string(),
        );
    }

    let (mut source_files, mut warnings) = resolve_source_files(&sources.include, root_dir)?;
    let main = match kind {
        ProjectKind::Program => {
            let main_raw = project_file.project.main.as_deref().ok_or_else(|| {
                "Program projects require `project.main`.\n  help: Set `main = \"src/main.fpas\"` in `[project]`."
                    .to_string()
            })?;
            let main_path = resolve_explicit_file_path("project.main", main_raw, root_dir)?;
            validate_source_extension(&main_path, "project.main")?;
            source_files.retain(|source| !same_file(source, &main_path));
            Some(main_path)
        }
        ProjectKind::Library => {
            if project_file.project.main.is_some() {
                return Err(
                    "Library projects must not define `project.main`.\n  help: Remove the `main` entry or change `project.kind` to `program`."
                        .to_string(),
                );
            }
            None
        }
    };

    if let Some(main_path) = main.as_deref() {
        validate_program_main_file(main_path, defines, &mut warnings)?;
    }

    source_files = validate_project_source_units(source_files, defines, &mut warnings)?;

    Ok(LoadedProject {
        kind,
        main,
        source_files,
        warnings,
    })
}

fn parse_project_kind(raw_kind: &str) -> Result<ProjectKind, String> {
    match raw_kind.trim() {
        "program" => Ok(ProjectKind::Program),
        "library" => Ok(ProjectKind::Library),
        other => Err(format!(
            "Invalid `project.kind` value `{other}`.\n  help: Use `program` or `library`."
        )),
    }
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "`{field_name}` must be a non-empty string.\n  help: Provide a value such as `\"my-app\"`."
        ));
    }

    Ok(())
}

fn validate_optional_non_empty(field_name: &str, value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_non_empty(field_name, value)?;
    }

    Ok(())
}

fn validate_program_main_file(
    main_path: &Path,
    defines: &DefineSet,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let (unit, parse_warnings) = parse_compilation_unit_file(main_path, defines)?;
    warnings.extend(parse_warnings);

    match unit {
        CompilationUnit::Program(_) => Ok(()),
        CompilationUnit::Unit(unit) => Err(format!(
            "`project.main` must declare `program`, but `{}` declares `unit {}`.\n  help: Use a `program` declaration in the main file.",
            main_path.to_string_lossy(),
            qualified_id_to_string(&unit.name)
        )),
    }
}

fn validate_project_source_units(
    source_files: Vec<PathBuf>,
    defines: &DefineSet,
    warnings: &mut Vec<String>,
) -> Result<Vec<PathBuf>, String> {
    let mut validated = Vec::new();
    let mut seen_unit_names = HashMap::<String, PathBuf>::new();

    for source_path in source_files {
        let (unit, parse_warnings) = parse_compilation_unit_file(&source_path, defines)?;
        warnings.extend(parse_warnings);

        match unit {
            CompilationUnit::Program(program) => {
                warnings.push(format!(
                    "Source file `{}` declares `program {}` and was skipped. Source files must use `unit` declarations.",
                    source_path.to_string_lossy(),
                    program.name
                ));
            }
            CompilationUnit::Unit(unit) => {
                validate_user_unit_name(&source_path, &unit.name)?;
                let unit_name = qualified_id_to_string(&unit.name);
                let key = unit_name.to_ascii_lowercase();
                if let Some(first_path) = seen_unit_names.get(&key) {
                    return Err(format!(
                        "Duplicate unit name `{unit_name}` found in `{}` and `{}`.\n  help: Use a unique `unit` namespace per source file.",
                        first_path.to_string_lossy(),
                        source_path.to_string_lossy()
                    ));
                }
                seen_unit_names.insert(key, source_path.clone());
                validated.push(source_path);
            }
        }
    }

    Ok(validated)
}

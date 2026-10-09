//! Shared compiled-unit project build path for `check` and `run`.
//!
//! Failures are coded records; see `docs/pascal/tools/diagnostics.md`.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::{
    INTERNAL_PROJECT_INVARIANT_FAILURE, PROJECT_MANIFEST_VALUE_INVALID, PROJECT_PATH_INVALID,
    PROJECT_SOURCE_READ_FAILED, PROJECT_UNIT_KIND_MISMATCH,
};
use fpas_project::{LoadedProject, ResolvedUnitGraph, StandardLibrary, UnitGraph};

use crate::cli_output::CliFailure;

pub(crate) struct ProjectProgram {
    pub(crate) executable: fpas_bytecode::VerifiedExecutable,
    pub(crate) source_paths: Vec<PathBuf>,
}

pub(crate) struct ProgramArtifact {
    pub(crate) path: PathBuf,
    pub(crate) reused: bool,
    pub(crate) executable: fpas_bytecode::VerifiedExecutable,
    pub(crate) source_paths: Vec<PathBuf>,
}

pub(crate) fn build_program(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, CliFailure> {
    let prepared = prepare_program(loaded, standard_library)?;
    let built = fpas_build::build_program(
        &prepared.graph,
        &prepared.selection,
        &prepared.program,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| CliFailure::from_build(&error, prepared.graph.source_paths()))?;
    Ok(ProjectProgram {
        executable: built.executable,
        source_paths: prepared.graph.source_paths().to_vec(),
    })
}

pub(crate) fn build_program_artifact(
    project_path: &Path,
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProgramArtifact, CliFailure> {
    let prepared = prepare_program(loaded, standard_library)?;
    let artifact_path = program_artifact_path(project_path, &loaded.name)?;
    let project_root = project_root(project_path)?;
    let source_paths = portable_source_paths(&prepared.graph, prepared.main, project_root)?;
    let built = fpas_build::build_program_artifact(
        &prepared.graph,
        fpas_build::ProgramArtifactTarget {
            path: &artifact_path,
            source: prepared.source.as_bytes(),
            source_paths: &source_paths,
        },
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| {
        CliFailure::from_build(&error, prepared.graph.source_paths())
            .with_actual_paths(&source_paths, prepared.graph.source_paths())
    })?;
    let reused = built.counters().program_image_reused == 1;
    Ok(ProgramArtifact {
        path: artifact_path,
        reused,
        executable: built.executable,
        source_paths: source_paths.into_iter().map(PathBuf::from).collect(),
    })
}

/// Builds a test entry with its own authoritative import-diagnostic path.
pub(crate) fn build_test_program(
    main: &Path,
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, CliFailure> {
    let (_, program) = parse_program(main)?;
    let graph = test_program_graph(main, source_files, link_meta, standard_library)?;
    build_graph_program(main, &graph, &program)
}

/// Builds one entry against shared units while retaining the entry's source identity.
pub(crate) fn build_test_program_with_graph(
    main: &Path,
    program_graph: &fpas_project::ProgramUnitGraph,
) -> Result<ProjectProgram, CliFailure> {
    let (_, program) = parse_program(main)?;
    let graph = program_graph.instantiate(main);
    build_graph_program(main, &graph, &program)
}

fn build_graph_program(
    main: &Path,
    graph: &UnitGraph,
    program: &fpas_parser::Program,
) -> Result<ProjectProgram, CliFailure> {
    let selection = fpas_project::resolve_program_units(graph, &program.uses, Some(main))?;
    let built = fpas_build::build_program(
        graph,
        &selection,
        program,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| CliFailure::from_build(&error, graph.source_paths()))?;
    Ok(ProjectProgram {
        executable: built.executable,
        source_paths: graph.source_paths().to_vec(),
    })
}

/// Checks a standalone program against sibling source units without publishing sidecars.
pub(crate) fn check_source_program(
    main: &Path,
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), CliFailure> {
    let (_, program) = parse_program(main)?;
    let graph = test_program_graph(main, source_files, link_meta, standard_library)?;
    let selection = fpas_project::resolve_program_units(&graph, &program.uses, Some(main))?;
    fpas_build::check_program(
        &graph,
        &selection,
        &program,
        &fpas_build::BuildOptions::default(),
    )
    .map(|_| ())
    .map_err(|error| CliFailure::from_build(&error, graph.source_paths()))
}

fn test_program_graph(
    main: &Path,
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<UnitGraph, CliFailure> {
    Ok(standard_library.map_or_else(
        || fpas_project::build_unit_graph_for_program(main, source_files, link_meta),
        |library| {
            fpas_project::build_unit_graph_for_program_with_standard_library(
                main,
                source_files,
                link_meta,
                library,
            )
        },
    )?)
}

pub(crate) fn check_library(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), CliFailure> {
    check_units(&loaded.source_files, &loaded.link_meta, standard_library)
}

pub(crate) fn check_test_project(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), CliFailure> {
    let unit_files = loaded
        .source_files
        .iter()
        .filter(|source| !fpas_project::is_test_source_file(source))
        .cloned()
        .collect::<Vec<_>>();

    if !unit_files.is_empty() {
        check_units(&unit_files, &loaded.link_meta, standard_library)?;
    }

    for test_path in loaded
        .source_files
        .iter()
        .filter(|source| fpas_project::is_test_source_file(source))
    {
        build_test_program(test_path, &unit_files, &loaded.link_meta, standard_library)?;
    }

    Ok(())
}

/// Builds library units and publishes their compiled-unit sidecars.
pub(crate) fn check_units(
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), CliFailure> {
    let graph = library_graph(source_files, link_meta, standard_library)?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    fpas_build::build_library_units(&graph, &selection, &fpas_build::BuildOptions::default())
        .map(|_| ())
        .map_err(|error| CliFailure::from_build(&error, graph.source_paths()))
}

/// Checks standalone source units without publishing sidecars.
pub(crate) fn check_source_units(
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), CliFailure> {
    let graph = library_graph(source_files, link_meta, standard_library)?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    fpas_build::check_library_units(&graph, &selection, &fpas_build::BuildOptions::default())
        .map(|_| ())
        .map_err(|error| CliFailure::from_build(&error, graph.source_paths()))
}

fn library_graph(
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<UnitGraph, CliFailure> {
    Ok(standard_library.map_or_else(
        || fpas_project::build_unit_graph(source_files, link_meta),
        |library| {
            fpas_project::build_unit_graph_with_standard_library(source_files, link_meta, library)
        },
    )?)
}

fn program_graph(
    main: &Path,
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<UnitGraph, CliFailure> {
    test_program_graph(
        main,
        &loaded.source_files,
        &loaded.link_meta,
        standard_library,
    )
}

struct PreparedProgram<'a> {
    main: &'a Path,
    source: String,
    program: fpas_parser::Program,
    graph: UnitGraph,
    selection: ResolvedUnitGraph,
}

fn prepare_program<'a>(
    loaded: &'a LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<PreparedProgram<'a>, CliFailure> {
    let main = loaded.main.as_deref().ok_or_else(missing_main)?;
    let (source, program) = parse_program(main)?;
    let graph = program_graph(main, loaded, standard_library)?;
    let selection = fpas_project::resolve_program_units(&graph, &program.uses, Some(main))?;
    Ok(PreparedProgram {
        main,
        source,
        program,
        graph,
        selection,
    })
}

/// Rejects a program project without `project.main`.
pub(crate) fn missing_main() -> CliFailure {
    CliFailure::new(
        PROJECT_MANIFEST_VALUE_INVALID,
        "Project is missing `project.main`.",
    )
    .with_help("Set `main = \"src/main.fpas\"` in `[project]`.")
}

/// Reports a source file that the CLI could not read.
pub(crate) fn source_read_failure(path: &Path, error: &std::io::Error) -> CliFailure {
    CliFailure::new(
        PROJECT_SOURCE_READ_FAILED,
        format!("Error reading source file: {error}"),
    )
    .with_help("Check that the source file exists and is readable.")
    .in_file(path)
}

fn parse_program(path: &Path) -> Result<(String, fpas_parser::Program), CliFailure> {
    let source = fs::read_to_string(path).map_err(|error| source_read_failure(path, &error))?;
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(&source);
    let errors = diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .filter(|diagnostic| diagnostic.is_error())
        .cloned()
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(CliFailure::from_diagnostics(path, &errors));
    }
    match unit {
        fpas_parser::CompilationUnit::Program(program) => Ok((source, program)),
        fpas_parser::CompilationUnit::Unit(unit) => Err(CliFailure::new(
            PROJECT_UNIT_KIND_MISMATCH,
            format!(
                "Main source declares unit `{}` instead of a program.",
                unit.name.parts.join(".")
            ),
        )
        .with_help("Use a `program` declaration in the main file.")
        .in_file(path)),
    }
}

fn program_artifact_path(project_path: &Path, project_name: &str) -> Result<PathBuf, CliFailure> {
    if !crate::artifact_filename::is_valid(project_name) {
        return Err(CliFailure::new(
            PROJECT_MANIFEST_VALUE_INVALID,
            format!("`project.name` `{project_name}` cannot be used as an artifact filename."),
        )
        .with_help(
            "Use a non-empty name without path separators or Windows-reserved filename syntax.",
        ));
    }
    Ok(project_root(project_path)?.join(format!("{project_name}.fpascp")))
}

fn project_root(project_path: &Path) -> Result<&Path, CliFailure> {
    project_path.parent().ok_or_else(|| {
        CliFailure::new(
            PROJECT_PATH_INVALID,
            format!(
                "Cannot resolve project root for `{}`.",
                project_path.display()
            ),
        )
        .with_help("Use a normal file path inside a directory.")
    })
}

fn portable_source_paths(
    graph: &UnitGraph,
    main: &Path,
    project_root: &Path,
) -> Result<Vec<String>, CliFailure> {
    graph
        .source_paths()
        .iter()
        .map(|path| portable_source_path(graph, path, main, project_root))
        .collect()
}

fn portable_source_path(
    graph: &UnitGraph,
    path: &Path,
    main: &Path,
    project_root: &Path,
) -> Result<String, CliFailure> {
    if let Ok(relative) = path.strip_prefix(project_root) {
        return Ok(normalize_source_path(relative));
    }
    let underivable = || {
        CliFailure::new(
            INTERNAL_PROJECT_INVARIANT_FAILURE,
            format!(
                "Cannot derive a diagnostic source name for `{}`.",
                path.display()
            ),
        )
    };
    if path == main {
        let file_name = path.file_name().ok_or_else(underivable)?;
        return Ok(format!("program/{}", file_name.to_string_lossy()));
    }
    let unit_name = graph
        .iter()
        .find_map(|(name, node)| (node.path() == path).then_some(name))
        .ok_or_else(underivable)?;
    Ok(format!("units/{}.fpas", unit_name.replace('.', "/")))
}

fn normalize_source_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

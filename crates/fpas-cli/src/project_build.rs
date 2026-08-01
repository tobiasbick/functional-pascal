//! Shared compiled-unit project build path for `check` and `run`.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_bytecode::Chunk;
use fpas_project::{LoadedProject, ResolvedUnitGraph, StandardLibrary, UnitGraph};

pub(crate) struct ProjectProgram {
    pub(crate) chunk: Chunk,
    pub(crate) source_paths: Vec<PathBuf>,
}

pub(crate) struct ProgramArtifact {
    pub(crate) path: PathBuf,
    pub(crate) reused: bool,
    pub(crate) chunk: Chunk,
    pub(crate) source_paths: Vec<PathBuf>,
}

pub(crate) fn build_program(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, String> {
    let prepared = prepare_program(loaded, standard_library)?;
    let built = fpas_build::build_program(
        &prepared.graph,
        &prepared.selection,
        &prepared.program,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| {
        format!(
            "Cannot build project `{}`: {error}",
            prepared.main.display()
        )
    })?;
    Ok(ProjectProgram {
        chunk: built.chunk,
        source_paths: prepared.graph.source_paths().to_vec(),
    })
}

pub(crate) fn build_program_artifact(
    project_path: &Path,
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProgramArtifact, String> {
    let prepared = prepare_program(loaded, standard_library)?;
    let artifact_path = program_artifact_path(project_path, &loaded.name)?;
    let project_root = project_path.parent().ok_or_else(|| {
        format!(
            "Cannot resolve project root for `{}`.",
            project_path.display()
        )
    })?;
    let source_paths = portable_source_paths(&prepared.graph, prepared.main, project_root)?;
    let built = fpas_build::build_program_artifact(
        &prepared.graph,
        &prepared.selection,
        &prepared.program,
        fpas_build::ProgramArtifactTarget {
            path: &artifact_path,
            source: prepared.source.as_bytes(),
            source_paths: &source_paths,
        },
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| {
        format!(
            "Cannot build program project `{}`: {error}",
            prepared.main.display()
        )
    })?;
    let reused = built.counters().program_image_reused == 1;
    Ok(ProgramArtifact {
        path: artifact_path,
        reused,
        chunk: built.chunk,
        source_paths: source_paths.into_iter().map(PathBuf::from).collect(),
    })
}

pub(crate) fn build_test_program(
    main: &Path,
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, String> {
    let (_, program) = parse_program(main)?;
    let graph = standard_library.map_or_else(
        || fpas_project::build_unit_graph_for_program(main, source_files, link_meta),
        |library| {
            fpas_project::build_unit_graph_for_program_with_standard_library(
                main,
                source_files,
                link_meta,
                library,
            )
        },
    )?;
    let selection = fpas_project::resolve_program_units(&graph, &program.uses)?;
    let built = fpas_build::build_program(
        &graph,
        &selection,
        &program,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| format!("Cannot build test program `{}`: {error}", main.display()))?;
    Ok(ProjectProgram {
        chunk: built.chunk,
        source_paths: graph.source_paths().to_vec(),
    })
}

pub(crate) fn check_library(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), String> {
    let graph = standard_library.map_or_else(
        || fpas_project::build_unit_graph(&loaded.source_files, &loaded.link_meta),
        |library| {
            fpas_project::build_unit_graph_with_standard_library(
                &loaded.source_files,
                &loaded.link_meta,
                library,
            )
        },
    )?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    fpas_build::build_library_units(&graph, &selection, &fpas_build::BuildOptions::default())
        .map(|_| ())
        .map_err(|error| format!("Cannot build library project: {error}"))
}

pub(crate) fn check_test_project(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), String> {
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

pub(crate) fn check_units(
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<(), String> {
    let graph = standard_library.map_or_else(
        || fpas_project::build_unit_graph(source_files, link_meta),
        |library| {
            fpas_project::build_unit_graph_with_standard_library(source_files, link_meta, library)
        },
    )?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    fpas_build::build_library_units(&graph, &selection, &fpas_build::BuildOptions::default())
        .map(|_| ())
        .map_err(|error| format!("Cannot build source units: {error}"))
}

fn program_graph(
    main: &Path,
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<UnitGraph, String> {
    standard_library.map_or_else(
        || {
            fpas_project::build_unit_graph_for_program(
                main,
                &loaded.source_files,
                &loaded.link_meta,
            )
        },
        |library| {
            fpas_project::build_unit_graph_for_program_with_standard_library(
                main,
                &loaded.source_files,
                &loaded.link_meta,
                library,
            )
        },
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
) -> Result<PreparedProgram<'a>, String> {
    let main = loaded
        .main
        .as_deref()
        .ok_or_else(|| "Project is missing `project.main`.".to_string())?;
    let (source, program) = parse_program(main)?;
    let graph = program_graph(main, loaded, standard_library)?;
    let selection = fpas_project::resolve_program_units(&graph, &program.uses)?;
    Ok(PreparedProgram {
        main,
        source,
        program,
        graph,
        selection,
    })
}

fn parse_program(path: &Path) -> Result<(String, fpas_parser::Program), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(&source);
    if let Some(diagnostic) = diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .find(|diagnostic| diagnostic.is_error())
    {
        return Err(fpas_diagnostics::render(
            &path.to_string_lossy(),
            diagnostic,
        ));
    }
    match unit {
        fpas_parser::CompilationUnit::Program(program) => Ok((source, program)),
        fpas_parser::CompilationUnit::Unit(unit) => Err(format!(
            "Main source `{}` declares unit `{}` instead of a program.",
            path.display(),
            unit.name.parts.join(".")
        )),
    }
}

fn program_artifact_path(project_path: &Path, project_name: &str) -> Result<PathBuf, String> {
    if matches!(project_name, "." | "..")
        || project_name.contains('/')
        || project_name.contains('\\')
    {
        return Err(format!(
            "`project.name` `{project_name}` cannot be used as an artifact filename.\n  help: Use a name without path separators."
        ));
    }
    let project_root = project_path.parent().ok_or_else(|| {
        format!(
            "Cannot resolve project root for `{}`.",
            project_path.display()
        )
    })?;
    Ok(project_root.join(format!("{project_name}.fpascp")))
}

fn portable_source_paths(
    graph: &UnitGraph,
    main: &Path,
    project_root: &Path,
) -> Result<Vec<String>, String> {
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
) -> Result<String, String> {
    if let Ok(relative) = path.strip_prefix(project_root) {
        return Ok(normalize_source_path(relative));
    }
    if path == main {
        let file_name = path.file_name().ok_or_else(|| {
            format!(
                "Cannot derive a diagnostic source name for `{}`.",
                path.display()
            )
        })?;
        return Ok(format!("program/{}", file_name.to_string_lossy()));
    }
    let unit_name = graph
        .iter()
        .find_map(|(name, node)| (node.path() == path).then_some(name))
        .ok_or_else(|| {
            format!(
                "Cannot derive a diagnostic source name for `{}`.",
                path.display()
            )
        })?;
    Ok(format!("units/{}.fpas", unit_name.replace('.', "/")))
}

fn normalize_source_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

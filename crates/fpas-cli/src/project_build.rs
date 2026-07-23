//! Shared compiled-unit project build path for `check` and `run`.

use std::fs;
use std::path::{Path, PathBuf};

use fpas_bytecode::Chunk;
use fpas_project::{LoadedProject, StandardLibrary, UnitGraph};

pub(crate) struct ProjectProgram {
    pub(crate) chunk: Chunk,
    pub(crate) source_paths: Vec<PathBuf>,
}

pub(crate) fn build_program(
    loaded: &LoadedProject,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, String> {
    let main = loaded
        .main
        .as_deref()
        .ok_or_else(|| "Project is missing `project.main`.".to_string())?;
    let program = parse_program(main)?;
    let graph = program_graph(main, loaded, standard_library)?;
    let selection = fpas_project::resolve_program_units(&graph, &program.uses)?;
    let built = fpas_build::build_program(
        &graph,
        &selection,
        &program,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| format!("Cannot build project `{}`: {error}", main.display()))?;
    Ok(ProjectProgram {
        chunk: built.chunk,
        source_paths: graph.source_paths().to_vec(),
    })
}

pub(crate) fn build_test_program(
    main: &Path,
    source_files: &[PathBuf],
    link_meta: &fpas_project::ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProjectProgram, String> {
    let program = parse_program(main)?;
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

fn parse_program(path: &Path) -> Result<fpas_parser::Program, String> {
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
        fpas_parser::CompilationUnit::Program(program) => Ok(program),
        fpas_parser::CompilationUnit::Unit(unit) => Err(format!(
            "Main source `{}` declares unit `{}` instead of a program.",
            path.display(),
            unit.name.parts.join(".")
        )),
    }
}

//! Conservative in-memory bundling for independent FPAS test programs.
//!
//! **Documentation:** `docs/pascal/std/testing/test.md`.

use std::path::PathBuf;

use fpas_parser::{Decl, FuncBody, ProcedureDecl, Program, Visibility};

use crate::{ProjectLinkMeta, StandardLibrary};

use super::parse::parse_program_file;
use super::source_map::apply_program_source_id;
use super::{build_link_environment, rewrite_main_program};

/// One linked program containing several independent test entry procedures.
pub struct LinkedTestBundle {
    /// Program passed through semantic analysis and code generation once.
    pub program: Program,
    /// Source paths indexed by source ID for diagnostics.
    pub source_paths: Vec<PathBuf>,
    /// Synthetic zero-argument procedure names in input order.
    pub entry_names: Vec<String>,
}

/// Links compatible test entry files once and returns one memory-only bundle.
pub fn build_test_bundle_from_paths(
    main_paths: &[PathBuf],
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<LinkedTestBundle, String> {
    build_test_bundle_from_paths_with_optional_standard_library(
        main_paths,
        source_files,
        link_meta,
        None,
    )
}

/// Links compatible test entry files once with implementation-owned standard-library sources.
pub fn build_test_bundle_from_paths_with_standard_library(
    main_paths: &[PathBuf],
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: &StandardLibrary,
) -> Result<LinkedTestBundle, String> {
    build_test_bundle_from_paths_with_optional_standard_library(
        main_paths,
        source_files,
        link_meta,
        Some(standard_library),
    )
}

fn build_test_bundle_from_paths_with_optional_standard_library(
    main_paths: &[PathBuf],
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<LinkedTestBundle, String> {
    if main_paths.len() < 2 {
        return Err("A test bundle requires at least two test entry files.".to_string());
    }

    let mut programs = Vec::with_capacity(main_paths.len());
    for (index, path) in main_paths.iter().enumerate() {
        let mut program = parse_program_file(path)?;
        if !program.declarations.is_empty() {
            return Err(format!(
                "Test program `{}` has module-level declarations and cannot be bundled.",
                path.display()
            ));
        }
        let source_id = u32::try_from(index).map_err(|_| {
            "Too many test entry files in bundle.\n  help: Reduce the bundle size.".to_string()
        })?;
        apply_program_source_id(&mut program, source_id);
        programs.push(program);
    }

    let expected_uses = canonical_uses(&programs[0]);
    if programs
        .iter()
        .skip(1)
        .any(|program| canonical_uses(program) != expected_uses)
    {
        return Err(
            "Test programs use different unit environments and cannot share a bundle.".to_string(),
        );
    }

    let mut source_paths = main_paths.to_vec();
    let environment = build_link_environment(
        &programs[0].uses,
        source_files,
        link_meta,
        standard_library,
        &mut source_paths,
    )?;
    for (program, path) in programs.iter_mut().zip(main_paths) {
        rewrite_main_program(program, path, &environment)?;
    }

    let first_span = programs[0].span;
    let uses = environment.std_uses.clone();
    let mut declarations = environment.unit_declarations;
    let mut entry_names = Vec::with_capacity(programs.len());
    for program in programs {
        append_test_entry(&mut declarations, &mut entry_names, program);
    }

    Ok(LinkedTestBundle {
        program: Program {
            name: "__fpas_test_image".to_string(),
            name_span: first_span,
            uses,
            declarations,
            body: Vec::new(),
            span: first_span,
        },
        source_paths,
        entry_names,
    })
}

fn canonical_uses(program: &Program) -> Vec<String> {
    program
        .uses
        .iter()
        .map(|used| used.parts.join(".").to_ascii_lowercase())
        .collect()
}

fn append_test_entry(
    declarations: &mut Vec<Decl>,
    entry_names: &mut Vec<String>,
    mut program: Program,
) {
    let entry_name = format!("__fpas_test_image_entry_{}", entry_names.len());
    let procedure = ProcedureDecl {
        name: entry_name.clone(),
        type_params: Vec::new(),
        params: Vec::new(),
        body: FuncBody::Block {
            nested: Vec::new(),
            stmts: std::mem::take(&mut program.body),
        },
        visibility: Visibility::Public,
        span: program.span,
    };
    declarations.push(Decl::Procedure(procedure));
    entry_names.push(entry_name);
}

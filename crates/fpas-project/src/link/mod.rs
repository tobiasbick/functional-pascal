mod graph;
mod import_policy;
mod imports;
mod library_check;
mod parse;
mod rewrite;
mod source_map;
mod support;

use crate::StandardLibrary;
use crate::common::qualified_id_to_string;
use crate::model::ProjectLinkMeta;
use graph::{resolve_reachable_units, topo_sort_units};
use import_policy::ImportPolicy;
use imports::{build_imports, collect_unit_symbol_maps};
use parse::{parse_program_file, parse_unit_files};
use rewrite::{NameRewriter, rename_top_level_decls};
use support::{collect_std_uses, internal_link_error, internal_symbol_error, merge_std_uses};

use fpas_parser::{Decl, Program, Unit};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct UnitFile {
    path: PathBuf,
    unit: Unit,
}

/// Fully linked project program together with its source-path table.
pub struct LinkedProgram {
    /// Linked main program with reachable unit declarations merged in.
    pub program: Program,
    /// Source paths indexed by source ID for diagnostic rendering.
    pub source_paths: Vec<PathBuf>,
}

/// Build a single linked `Program` from a main file plus project units.
///
/// This resolves reachable units, checks import ambiguity, preserves private
/// unit members, and rewrites user-unit symbols into fully qualified names as
/// described in `docs/pascal/program-structure/units.md`.
pub fn build_program(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<Program, String> {
    Ok(build_program_with_source_map(main_path, source_files, link_meta)?.program)
}

pub use library_check::{
    build_library_check_with_source_map, build_library_check_with_standard_library,
};

/// Build a single linked `Program` together with the source-path table used to
/// resolve diagnostics back to the originating file.
///
/// This resolves reachable units, checks import ambiguity, preserves private
/// unit members, and rewrites user-unit symbols into fully qualified names as
/// described in `docs/pascal/program-structure/units.md`.
pub fn build_program_with_source_map(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<LinkedProgram, String> {
    build_program_with_optional_standard_library(main_path, source_files, link_meta, None)
}

/// Builds a program while making implementation-owned standard-library sources available.
pub fn build_program_with_standard_library(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: &StandardLibrary,
) -> Result<LinkedProgram, String> {
    build_program_with_optional_standard_library(
        main_path,
        source_files,
        link_meta,
        Some(standard_library),
    )
}

fn build_program_with_optional_standard_library(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<LinkedProgram, String> {
    let mut main_program = parse_program_file(main_path)?;

    let mut source_paths = vec![main_path.to_path_buf()];
    let standard_source_files = standard_library.map_or(&[][..], StandardLibrary::source_files);
    let mut all_source_files = source_files.to_vec();
    all_source_files.extend_from_slice(standard_source_files);
    let standard_source_paths = standard_source_files.iter().collect();
    let units = parse_unit_files(&all_source_files, &mut source_paths, &standard_source_paths)?;
    let import_policy = ImportPolicy::new(link_meta, &units);
    import_policy.validate_root_uses(&main_program.uses)?;

    let reachable_unit_keys = resolve_reachable_units(&main_program.uses, &units, &import_policy)?;
    let unit_order = topo_sort_units(&reachable_unit_keys, &units)?;
    let (exports, all_symbols) = collect_unit_symbol_maps(&reachable_unit_keys, &units)?;

    let canonical_units: std::collections::HashMap<String, Vec<String>> = units
        .iter()
        .map(|(key, uf)| (key.clone(), uf.unit.name.parts.clone()))
        .collect();

    let mut std_uses = collect_std_uses(&main_program.uses);
    let mut merged_unit_decls = Vec::<Decl>::new();

    for unit_key in unit_order {
        let Some(unit_file) = units.get(&unit_key) else {
            return Err(internal_link_error(
                &unit_key,
                "merging units after topological sorting",
            ));
        };
        merge_std_uses(&mut std_uses, &unit_file.unit.uses);

        let unit_name = qualified_id_to_string(&unit_file.unit.name);
        let Some(own_symbols) = all_symbols.get(&unit_key) else {
            return Err(internal_symbol_error(&unit_key));
        };
        let imports = build_imports(
            &unit_key,
            &unit_file.unit.uses,
            Some(own_symbols),
            &exports,
            &units,
            &import_policy,
        )?;

        let mut declarations = unit_file.unit.declarations.clone();
        rename_top_level_decls(&mut declarations, &unit_name);

        let mut rewriter = NameRewriter::new(
            unit_file.path.to_string_lossy().into_owned(),
            &imports.resolved,
            &imports.ambiguous,
            &canonical_units,
        );
        rewriter.rewrite_declarations(&mut declarations);
        rewriter.raise_first_error()?;

        merged_unit_decls.extend(declarations);
    }

    let main_imports = build_imports(
        "__main__",
        &main_program.uses,
        None,
        &exports,
        &units,
        &import_policy,
    )?;
    let mut main_rewriter = NameRewriter::new(
        main_path.to_string_lossy().into_owned(),
        &main_imports.resolved,
        &main_imports.ambiguous,
        &canonical_units,
    );
    main_rewriter.rewrite_declarations(&mut main_program.declarations);
    main_rewriter.rewrite_statements(&mut main_program.body);
    main_rewriter.raise_first_error()?;

    main_program.uses = std_uses;
    merged_unit_decls.append(&mut main_program.declarations);
    main_program.declarations = merged_unit_decls;
    Ok(LinkedProgram {
        program: main_program,
        source_paths,
    })
}

//! Links all library units into a stub program for type-checking.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`

use super::LinkedProgram;
use super::graph::{collect_library_reachable_units, topo_sort_units};
use super::imports::{build_imports, collect_unit_symbol_maps};
use super::parse::parse_unit_files;
use super::rewrite::{NameRewriter, rename_top_level_decls};
use super::support::{
    collect_std_uses, internal_link_error, internal_symbol_error, merge_std_uses,
};
use crate::common::qualified_id_to_string;
use crate::model::ProjectLinkMeta;

use fpas_parser::Decl;
use std::collections::HashMap;
use std::path::PathBuf;

const LIBRARY_CHECK_SOURCE: &str = "program __FpasLibraryCheck;\nbegin\nend.\n";

/// Build a linked stub `Program` that contains every project unit for semantic checking.
///
/// Documentation: `docs/pascal/program-structure/projects.md`
pub fn build_library_check_with_source_map(
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<LinkedProgram, String> {
    let (mut main_program, parse_errors) = fpas_parser::parse(LIBRARY_CHECK_SOURCE);
    let has_errors = parse_errors
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().is_error());
    if has_errors {
        return Err("Internal error: failed to parse the library check stub program.".to_string());
    }

    let mut source_paths = Vec::new();
    let units = parse_unit_files(source_files, &mut source_paths)?;
    let import_policy = super::import_policy::ImportPolicy::new(link_meta, &units);

    let reachable_unit_keys = collect_library_reachable_units(&units)?;
    let unit_order = topo_sort_units(&reachable_unit_keys, &units)?;
    let (exports, all_symbols) = collect_unit_symbol_maps(&reachable_unit_keys, &units)?;

    let canonical_units: HashMap<String, Vec<String>> = units
        .iter()
        .map(|(key, unit_file)| (key.clone(), unit_file.unit.name.parts.clone()))
        .collect();

    let mut std_uses = collect_std_uses(&main_program.uses);
    let mut merged_unit_decls = Vec::<Decl>::new();

    for unit_key in unit_order {
        let Some(unit_file) = units.get(&unit_key) else {
            return Err(internal_link_error(
                &unit_key,
                "merging library units after topological sorting",
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

    main_program.uses = std_uses;
    main_program.declarations = merged_unit_decls;
    main_program.body.clear();

    Ok(LinkedProgram {
        program: main_program,
        source_paths,
    })
}

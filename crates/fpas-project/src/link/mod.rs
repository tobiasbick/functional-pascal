mod graph;
mod imports;
mod parse;
mod rewrite;
mod support;

use crate::common::qualified_id_to_string;
use graph::{resolve_reachable_units, topo_sort_units};
use imports::{build_imports, collect_all_unit_symbols, collect_unit_exports};
use parse::{parse_program_file, parse_unit_files};
use rewrite::{NameRewriter, rename_top_level_decls};
use support::{collect_std_uses, internal_link_error, internal_symbol_error, merge_std_uses};

use fpas_parser::{Decl, Program, Unit};
use std::path::{Path, PathBuf};

struct UnitFile {
    path: PathBuf,
    unit: Unit,
}

/// Build a single linked `Program` from a main file plus project units.
///
/// This resolves reachable units, checks import ambiguity, preserves private
/// unit members, and rewrites user-unit symbols into fully qualified names as
/// described in `docs/pascal/09-units.md`.
pub fn build_program(main_path: &Path, source_files: &[PathBuf]) -> Result<Program, String> {
    let mut main_program = parse_program_file(main_path)?;
    let units = parse_unit_files(source_files)?;

    let reachable_unit_keys = resolve_reachable_units(&main_program.uses, &units)?;
    let unit_order = topo_sort_units(&reachable_unit_keys, &units)?;
    let exports = collect_unit_exports(&reachable_unit_keys, &units)?;
    let all_symbols = collect_all_unit_symbols(&reachable_unit_keys, &units)?;

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
        let imports = build_imports(&unit_file.unit.uses, Some(own_symbols), &exports, &units)?;

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

    let main_imports = build_imports(&main_program.uses, None, &exports, &units)?;
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
    Ok(main_program)
}

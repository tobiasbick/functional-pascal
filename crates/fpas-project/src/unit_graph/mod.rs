//! Parsed unit dependency graphs for source linking and compiled-unit builds.
//!
//! Documentation: `docs/pascal/program-structure/units.md` and
//! `docs/pascal/program-structure/projects.md`.

mod model;
mod order;
mod parsed;
mod resolve;
mod source_map;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fpas_parser::{CompilationUnit, QualifiedId, Unit};

use crate::model::ProjectLinkMeta;
use crate::source::{
    display_unit_key, qualified_id_to_string, read_compilation_unit_file, validate_user_unit_name,
};
use crate::{StandardLibrary, is_test_source_file};
use source_map::apply_unit_source_id;

pub use model::{ResolvedUnitGraph, UnitGraph, UnitNode};
pub use parsed::{
    build_unit_graph_for_program_from_parsed_sources, build_unit_graph_from_parsed_sources,
};
pub(crate) use resolve::ImportPolicy;

use order::resolve_order;
use resolve::{all_library_units, resolve_reachable};

/// Parses project units and returns a graph independent from source declaration merging.
pub fn build_unit_graph(
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<UnitGraph, String> {
    build_unit_graph_with_base(source_files, link_meta, None, Vec::new())
}

/// Parses project and source standard-library units into one dependency graph.
pub fn build_unit_graph_with_standard_library(
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: &StandardLibrary,
) -> Result<UnitGraph, String> {
    build_unit_graph_with_base(source_files, link_meta, Some(standard_library), Vec::new())
}

/// Builds a program unit graph whose source table reserves ID zero for the main file.
pub fn build_unit_graph_for_program(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
) -> Result<UnitGraph, String> {
    build_unit_graph_with_base(source_files, link_meta, None, vec![main_path.to_path_buf()])
}

/// Builds a program unit graph including source-defined standard-library units.
pub fn build_unit_graph_for_program_with_standard_library(
    main_path: &Path,
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: &StandardLibrary,
) -> Result<UnitGraph, String> {
    build_unit_graph_with_base(
        source_files,
        link_meta,
        Some(standard_library),
        vec![main_path.to_path_buf()],
    )
}

pub(crate) fn build_unit_graph_with_base(
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
    mut source_paths: Vec<PathBuf>,
) -> Result<UnitGraph, String> {
    let effective_link_meta = merge_standard_library_link_meta(link_meta, standard_library);
    let mut nodes = HashMap::<String, UnitNode>::new();

    for source_path in source_files {
        let (source, parsed, _) = read_compilation_unit_file(source_path, 0)?;
        let CompilationUnit::Unit(unit) = parsed else {
            if is_test_source_file(source_path) {
                continue;
            }
            let CompilationUnit::Program(program) = parsed else {
                unreachable!("compilation unit is program or unit");
            };
            return Err(format!(
                "Source file `{}` declares `program {}`. Source files must use `unit` declarations.",
                source_path.display(),
                program.name
            ));
        };
        insert_unit(
            &mut nodes,
            &mut source_paths,
            &effective_link_meta,
            source_path,
            unit,
            Some(fpas_unit::Digest::of(&source)),
            true,
        )?;
    }

    if let Some(standard_library) = standard_library {
        for source_path in standard_library.source_files() {
            let (source, parsed, _) = read_compilation_unit_file(source_path, 0)?;
            let CompilationUnit::Unit(unit) = parsed else {
                return Err(format!(
                    "Standard library source file `{}` must declare a unit.",
                    source_path.display()
                ));
            };
            insert_unit(
                &mut nodes,
                &mut source_paths,
                &effective_link_meta,
                source_path,
                unit,
                Some(fpas_unit::Digest::of(&source)),
                false,
            )?;
        }
    }

    Ok(UnitGraph::new(nodes, effective_link_meta, source_paths))
}

fn merge_standard_library_link_meta(
    link_meta: &ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> ProjectLinkMeta {
    let mut combined = link_meta.clone();
    let Some(standard_library) = standard_library else {
        return combined;
    };
    combined
        .source_origins
        .extend(standard_library.link_meta().source_origins.clone());
    combined
        .library_export_policies
        .extend(standard_library.link_meta().library_export_policies.clone());
    combined.trusted_standard_library_sources.extend(
        standard_library
            .link_meta()
            .trusted_standard_library_sources
            .clone(),
    );
    combined
}

fn insert_unit(
    nodes: &mut HashMap<String, UnitNode>,
    source_paths: &mut Vec<PathBuf>,
    link_meta: &ProjectLinkMeta,
    source_path: &Path,
    mut unit: Unit,
    source_hash: Option<fpas_unit::Digest>,
    validate_name: bool,
) -> Result<(), String> {
    let source_id = next_source_id(source_paths.len())?;
    source_paths.push(source_path.to_path_buf());
    apply_unit_source_id(&mut unit, source_id);
    if validate_name {
        validate_user_unit_name(source_path, &unit.name)?;
    }

    let key = canonical_unit_key(&unit.name);
    if let Some(existing) = nodes.get(&key) {
        return Err(format!(
            "Duplicate unit name `{}` found in `{}` and `{}`.\n  help: Use unique unit names across source files.",
            qualified_id_to_string(&unit.name),
            existing.path().display(),
            source_path.display()
        ));
    }

    nodes.insert(
        key,
        UnitNode::new(
            source_path.to_path_buf(),
            link_meta.origin_for_source(source_path),
            unit,
            source_hash,
        ),
    );
    Ok(())
}

/// Resolves units reachable from a program or test entry `uses` clause.
pub fn resolve_program_units(
    graph: &UnitGraph,
    root_uses: &[QualifiedId],
) -> Result<ResolvedUnitGraph, String> {
    let policy = ImportPolicy::new(graph);
    let reachable = resolve_reachable(root_uses, graph, &policy)?;
    resolve_order(&reachable, graph)
}

/// Resolves every unit in a library in stable dependency-first order.
pub fn resolve_library_units(graph: &UnitGraph) -> Result<ResolvedUnitGraph, String> {
    let reachable = all_library_units(graph)?;
    resolve_order(&reachable, graph)
}

pub(crate) fn canonical_unit_key(id: &QualifiedId) -> String {
    qualified_id_to_string(id).to_ascii_lowercase()
}

pub(crate) fn is_intrinsic_std_unit(used: &QualifiedId, graph: &UnitGraph) -> bool {
    let key = canonical_unit_key(used);
    !graph.contains(&key)
        && fpas_std::STD_UNITS_INTRINSIC
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&key))
}

pub(crate) fn internal_graph_error(unit_key: &str, context: &str) -> String {
    format!(
        "Internal unit graph error: unit `{}` disappeared while {context}.\n  help: This indicates inconsistent project graph construction.",
        display_unit_key(unit_key)
    )
}

pub(crate) fn unknown_unit_error(key: &str, graph: &UnitGraph, owner: &str) -> String {
    let mut known = graph
        .iter()
        .map(|(_, node)| node.display_name().to_string())
        .collect::<Vec<_>>();
    known.sort();
    let display = display_unit_key(key);
    if known.is_empty() {
        format!(
            "Unknown unit `{display}` in {owner}. No source units are available in the project."
        )
    } else {
        format!(
            "Unknown unit `{display}` in {owner}.\n  help: Available units: {}.",
            known.join(", ")
        )
    }
}

fn next_source_id(source_path_count: usize) -> Result<u32, String> {
    u32::try_from(source_path_count).map_err(|_| {
        format!(
            "Too many source files in project: {source_path_count}.
  help: Reduce the number of linked source files so source IDs fit into 32 bits."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::next_source_id;

    #[test]
    fn source_id_rejects_counts_above_u32() {
        let result = next_source_id((u32::MAX as usize).saturating_add(1));

        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Too many source files in project")
        );
    }
}

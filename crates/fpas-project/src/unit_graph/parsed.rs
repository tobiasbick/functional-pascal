//! Unit-graph construction from caller-owned parsed source snapshots.

use std::path::{Path, PathBuf};

use fpas_parser::Unit;

use super::{UnitGraph, insert_unit};
use crate::ProjectLinkMeta;

/// Builds a unit graph from parsed source snapshots without reading sidecars or source files.
///
/// This is the overlay-safe counterpart to [`super::build_unit_graph`]. Callers retain ownership
/// of the authoritative source text and pass one parsed [`Unit`] for each project source.
pub fn build_unit_graph_from_parsed_sources(
    sources: Vec<(PathBuf, Unit)>,
    link_meta: &ProjectLinkMeta,
) -> Result<UnitGraph, String> {
    build_from_parsed_sources(sources, link_meta, Vec::new())
}

/// Builds a program unit graph from parsed snapshots, reserving source ID zero for the main file.
///
/// This function performs no filesystem writes and does not inspect `.fpascu` sidecars.
pub fn build_unit_graph_for_program_from_parsed_sources(
    main_path: &Path,
    sources: Vec<(PathBuf, Unit)>,
    link_meta: &ProjectLinkMeta,
) -> Result<UnitGraph, String> {
    build_from_parsed_sources(sources, link_meta, vec![main_path.to_path_buf()])
}

fn build_from_parsed_sources(
    sources: Vec<(PathBuf, Unit)>,
    link_meta: &ProjectLinkMeta,
    mut source_paths: Vec<PathBuf>,
) -> Result<UnitGraph, String> {
    let mut nodes = std::collections::HashMap::new();
    for (path, unit) in sources {
        let validate_name = !link_meta.is_trusted_standard_library_source(&path);
        insert_unit(
            &mut nodes,
            &mut source_paths,
            link_meta,
            &path,
            unit,
            validate_name,
        )?;
    }
    Ok(UnitGraph::new(nodes, link_meta.clone(), source_paths))
}

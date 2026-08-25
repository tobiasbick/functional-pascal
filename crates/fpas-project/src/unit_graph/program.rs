//! Reusable unit-graph snapshots for compiling multiple program entry points.

use std::path::{Path, PathBuf};

use super::{UnitGraph, build_unit_graph_with_base};
use crate::{ProjectLinkMeta, StandardLibrary};

/// Parsed unit graph whose source ID zero can be assigned to multiple program entries.
///
/// The graph is an immutable snapshot of the supplied unit sources. It is intended for one build
/// operation that compiles several programs against the same project and standard-library state.
#[derive(Debug, Clone)]
pub struct ProgramUnitGraph {
    template: UnitGraph,
}

impl ProgramUnitGraph {
    /// Creates a cheap graph instance whose source ID zero names `main_path`.
    #[must_use]
    pub fn instantiate(&self, main_path: &Path) -> UnitGraph {
        self.template.with_program_source_path(main_path)
    }
}

/// Parses one reusable program-unit graph from project and standard-library sources.
///
/// # Errors
///
/// Returns an error when a source cannot be read or parsed, declares an invalid or duplicate unit,
/// or violates project ownership rules.
pub fn prepare_program_unit_graph(
    source_files: &[PathBuf],
    link_meta: &ProjectLinkMeta,
    standard_library: Option<&StandardLibrary>,
) -> Result<ProgramUnitGraph, String> {
    let template = build_unit_graph_with_base(
        source_files,
        link_meta,
        standard_library,
        vec![PathBuf::new()],
    )?;
    Ok(ProgramUnitGraph { template })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_graph_assigns_each_program_its_own_source_path() {
        let prepared = prepare_program_unit_graph(&[], &ProjectLinkMeta::default(), None)
            .expect("empty program graph must build");

        let first = prepared.instantiate(Path::new("first.fpas"));
        let second = prepared.instantiate(Path::new("second.fpas"));

        assert_eq!(first.source_paths(), &[PathBuf::from("first.fpas")]);
        assert_eq!(second.source_paths(), &[PathBuf::from("second.fpas")]);
    }
}

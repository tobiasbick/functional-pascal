//! Fresh standard-library compilation for distribution staging.

mod publication;
mod tree;

use std::fmt;
use std::path::Path;

use crate::{BuildCounters, BuildOptions, build_library_units};

/// Failure while preparing a complete standard-library distribution tree.
#[derive(Debug)]
pub struct DistributionError {
    detail: String,
}

impl DistributionError {
    fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for DistributionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for DistributionError {}

/// Recompiles every standard-library unit and exactly replaces the delivered tree.
///
/// The source root is a disposable staging tree. Existing compiled-unit files are
/// removed before graph loading so every sidecar is produced by the current compiler.
/// Symbolic links and reparse points in that tree are rejected before cleanup or copying.
pub fn stage_standard_library(
    source_root: &Path,
    destination_root: &Path,
    options: &BuildOptions,
) -> Result<BuildCounters, DistributionError> {
    tree::validate_separate_trees(source_root, destination_root)
        .map_err(|error| DistributionError::new(error.to_string()))?;

    tree::remove_compiled_unit_artifacts(source_root).map_err(|error| {
        DistributionError::new(format!(
            "cannot clean standard-library staging directory `{}`: {error}",
            source_root.display()
        ))
    })?;
    let library =
        fpas_project::load_standard_library(source_root).map_err(DistributionError::new)?;
    let graph = fpas_project::build_unit_graph_with_standard_library(
        &[],
        &fpas_project::ProjectLinkMeta::default(),
        &library,
    )
    .map_err(DistributionError::new)?;
    let selection = fpas_project::resolve_library_units(&graph).map_err(DistributionError::new)?;
    let built = build_library_units(&graph, &selection, options)
        .map_err(|error| DistributionError::new(error.to_string()))?;
    let counters = built.counters();

    publication::replace_tree(source_root, destination_root).map_err(|error| {
        DistributionError::new(format!(
            "cannot replace standard-library distribution directory `{}`: {error}",
            destination_root.display()
        ))
    })?;
    Ok(counters)
}

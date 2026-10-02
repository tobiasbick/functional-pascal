//! Fresh standard-library compilation for distribution staging.

mod publication;
mod tree;

use std::io;
use std::path::Path;

use fpas_diagnostics::codes::{BUILD_ARTIFACT_IO_FAILED, PROJECT_PATH_INVALID};

use crate::{BuildCounters, BuildError, BuildOptions, build_library_units};

/// Recompiles every standard-library unit and exactly replaces the delivered tree.
///
/// The source root is a disposable staging tree. Existing compiled-unit files are
/// removed before graph loading so every sidecar is produced by the current compiler.
/// Symbolic links and reparse points in that tree are rejected before cleanup or copying.
///
/// # Errors
/// Returns coded records for rejected directory layouts, filesystem failures,
/// standard-library project errors and unit compilation failures.
pub fn stage_standard_library(
    source_root: &Path,
    destination_root: &Path,
    options: &BuildOptions,
) -> Result<BuildCounters, BuildError> {
    tree::validate_separate_trees(source_root, destination_root)
        .map_err(|error| distribution_error("invalid distribution directories".into(), &error))?;

    tree::remove_compiled_unit_artifacts(source_root).map_err(|error| {
        distribution_error(
            format!(
                "cannot clean standard-library staging directory `{}`",
                source_root.display()
            ),
            &error,
        )
    })?;
    let library = fpas_project::load_standard_library(source_root)?;
    let graph = fpas_project::build_unit_graph_with_standard_library(
        &[],
        &fpas_project::ProjectLinkMeta::default(),
        &library,
    )?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    let built = build_library_units(&graph, &selection, options)?;
    let counters = built.counters();

    publication::replace_tree(source_root, destination_root).map_err(|error| {
        distribution_error(
            format!(
                "cannot replace standard-library distribution directory `{}`",
                destination_root.display()
            ),
            &error,
        )
    })?;
    Ok(counters)
}

// Rejected layouts and links are path errors; other filesystem failures are artifact I/O.
fn distribution_error(context: String, error: &io::Error) -> BuildError {
    let code = if error.kind() == io::ErrorKind::InvalidInput {
        PROJECT_PATH_INVALID
    } else {
        BUILD_ARTIFACT_IO_FAILED
    };
    BuildError::new(code, format!("{context}: {error}"))
}

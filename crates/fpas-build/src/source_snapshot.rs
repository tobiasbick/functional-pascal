//! Immutable unit-source snapshots used by incremental compilation.

use std::fs;
use std::io;
use std::path::Path;

use fpas_diagnostics::codes::{
    INTERNAL_PROJECT_INVARIANT_FAILURE, PROJECT_SOURCE_CHANGED, PROJECT_SOURCE_READ_FAILED,
};
use fpas_project::UnitNode;
use fpas_unit::Digest;

use crate::BuildError;

pub(crate) struct UnitSourceSnapshot {
    bytes: Vec<u8>,
    hash: Digest,
}

impl UnitSourceSnapshot {
    pub(crate) fn read(node: &UnitNode) -> Result<Self, BuildError> {
        let graph_hash = node.source_hash().ok_or_else(|| {
            BuildError::new(
                INTERNAL_PROJECT_INVARIANT_FAILURE,
                format!(
                    "cannot compile unit `{}` from a parsed overlay graph without an authoritative source snapshot",
                    node.display_name()
                ),
            )
        })?;
        let bytes = read_source(node)?;
        let hash = Digest::of(&bytes);
        if hash != graph_hash {
            return Err(changed_source_error(node));
        }
        Ok(Self { bytes, hash })
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn hash(&self) -> Digest {
        self.hash
    }

    pub(crate) fn ensure_current(&self, node: &UnitNode) -> Result<(), BuildError> {
        if Digest::of(read_source(node)?) != self.hash {
            return Err(changed_source_error(node));
        }
        Ok(())
    }
}

fn read_source(node: &UnitNode) -> Result<Vec<u8>, BuildError> {
    fs::read(node.path()).map_err(|error| source_read_error(node.path(), &error))
}

fn changed_source_error(node: &UnitNode) -> BuildError {
    source_changed_error(node.path(), "after the build graph was created")
}

/// Reports a source file that could not be read while building.
pub(crate) fn source_read_error(path: &Path, error: &io::Error) -> BuildError {
    BuildError::new(
        PROJECT_SOURCE_READ_FAILED,
        format!("Error reading source file: {error}"),
    )
    .with_help("Check that the source file exists and is readable.")
    .in_source(path)
}

/// Reports a source file whose bytes no longer match the build snapshot.
pub(crate) fn source_changed_error(path: &Path, when: &str) -> BuildError {
    BuildError::new(
        PROJECT_SOURCE_CHANGED,
        format!("Source file changed {when}."),
    )
    .with_help("Reload the project and retry the build.")
    .in_source(path)
}

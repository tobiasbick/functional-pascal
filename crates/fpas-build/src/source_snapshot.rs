//! Immutable unit-source snapshots used by incremental compilation.

use std::fs;

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
            BuildError::new(format!(
                "cannot compile unit `{}` from a parsed overlay graph without an authoritative source snapshot",
                node.display_name()
            ))
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
    fs::read(node.path()).map_err(|error| {
        BuildError::new(format!(
            "cannot read unit source `{}`: {error}",
            node.path().display()
        ))
    })
}

fn changed_source_error(node: &UnitNode) -> BuildError {
    BuildError::new(format!(
        "unit source `{}` changed after the build graph was created\n  help: Reload the project and retry the build.",
        node.path().display()
    ))
}

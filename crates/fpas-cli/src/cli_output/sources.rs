//! Compilation source identities retained through runtime and worker transport.

use std::path::{Path, PathBuf};

use fpas_program::Digest;

/// A diagnostic label tied to compilation text or a verified source hash.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct RuntimeSource {
    path: PathBuf,
    label: PathBuf,
    hash: [u8; 32],
    text: Option<String>,
}

impl RuntimeSource {
    /// Retains a root source snapshot already read for compilation.
    pub(crate) fn snapshot(path: &Path, text: &str) -> Self {
        let mut source = Self::hashed(path, Digest::of(text.as_bytes()));
        source.text = Some(text.to_owned());
        source
    }

    /// Records a source identity whose text must be hash-checked before display.
    pub(crate) fn hashed(path: &Path, hash: Digest) -> Self {
        Self {
            path: path.to_path_buf(),
            label: path.to_path_buf(),
            hash: *hash.as_bytes(),
            text: None,
        }
    }

    /// Keeps the root snapshot and graph unit hashes from the compilation input.
    pub(crate) fn graph(graph: &fpas_project::UnitGraph, main: &Path, text: &str) -> Vec<Self> {
        let mut sources = vec![Self::snapshot(main, text)];
        sources.extend(
            graph
                .iter()
                .filter_map(|(_, node)| Some(Self::hashed(node.path(), node.source_hash()?))),
        );
        sources
    }

    /// Replaces an actual source path's display label with its compiled-image identity.
    pub(crate) fn relabel(&mut self, actual: &[PathBuf], labels: &[String]) {
        if let Some(index) = actual.iter().position(|path| path == &self.path)
            && let Some(label) = labels.get(index)
        {
            self.label = PathBuf::from(label);
        }
    }

    /// Returns the diagnostic path under which the snapshot must be cached.
    pub(crate) fn label(&self) -> &Path {
        &self.label
    }

    /// Reads only the exact known path and rejects content from another compilation.
    pub(crate) fn text(&self) -> Option<String> {
        if let Some(text) = &self.text {
            return Some(text.clone());
        }
        let text = std::fs::read_to_string(&self.path).ok()?;
        (Digest::of(text.as_bytes()).as_bytes() == &self.hash).then_some(text)
    }
}

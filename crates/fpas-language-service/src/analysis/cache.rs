//! Analysis cache keyed by every participating immutable source version.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::document::{DocumentSnapshot, SourceVersion, normalized_path};

use super::DocumentAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct AnalysisFingerprint {
    identity: PathBuf,
    versions: Vec<(PathBuf, SourceVersion)>,
}

impl AnalysisFingerprint {
    pub(super) fn new(identity: &Path, snapshots: &[Arc<DocumentSnapshot>]) -> Self {
        let mut versions = snapshots
            .iter()
            .map(|snapshot| (snapshot.path().to_path_buf(), snapshot.version()))
            .collect::<Vec<_>>();
        versions.sort_by(|left, right| left.0.cmp(&right.0));
        Self {
            identity: normalized_path(identity),
            versions,
        }
    }

    fn identity(&self) -> &Path {
        &self.identity
    }
}

pub(super) struct AnalysisSet {
    documents: HashMap<PathBuf, Arc<DocumentAnalysis>>,
}

impl AnalysisSet {
    pub(super) fn new(documents: HashMap<PathBuf, Arc<DocumentAnalysis>>) -> Self {
        Self { documents }
    }

    pub(super) fn one(analysis: Arc<DocumentAnalysis>) -> Self {
        Self::new(HashMap::from([(
            analysis.snapshot().path().to_path_buf(),
            analysis,
        )]))
    }

    pub(super) fn document(&self, path: &Path) -> Option<Arc<DocumentAnalysis>> {
        self.documents.get(&normalized_path(path)).cloned()
    }
}

#[derive(Default)]
pub(super) struct AnalysisCache {
    entries: HashMap<AnalysisFingerprint, Arc<AnalysisSet>>,
}

impl AnalysisCache {
    pub(super) fn get(&self, fingerprint: &AnalysisFingerprint) -> Option<Arc<AnalysisSet>> {
        self.entries.get(fingerprint).cloned()
    }

    pub(super) fn insert(
        &mut self,
        fingerprint: AnalysisFingerprint,
        analysis: AnalysisSet,
    ) -> Arc<AnalysisSet> {
        self.entries
            .retain(|key, _| key.identity() != fingerprint.identity());
        let analysis = Arc::new(analysis);
        self.entries.insert(fingerprint, Arc::clone(&analysis));
        analysis
    }

    pub(super) fn invalidate_identities(
        &mut self,
        identities: &std::collections::BTreeSet<PathBuf>,
    ) {
        self.entries
            .retain(|fingerprint, _| !identities.contains(fingerprint.identity()));
    }

    pub(super) fn invalidate_path(&mut self, path: &Path) {
        let path = normalized_path(path);
        self.entries.retain(|fingerprint, _| {
            fingerprint.identity() != path
                && !fingerprint
                    .versions
                    .iter()
                    .any(|(source, _)| *source == path)
        });
    }
}

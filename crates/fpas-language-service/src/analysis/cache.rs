//! Analysis cache keyed by every participating immutable source version.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::document::{DocumentSnapshot, normalized_path};

use super::DocumentAnalysis;

/// Complete source and project identity needed to reuse semantic metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AnalysisFingerprint {
    identity: PathBuf,
    revisions: Vec<(PathBuf, u64)>,
    project: Option<crate::ProjectContext>,
}

impl AnalysisFingerprint {
    /// Identifies both immutable source versions and their semantic project context.
    pub(super) fn new(
        identity: &Path,
        snapshots: &[Arc<DocumentSnapshot>],
        project: Option<&crate::ProjectContext>,
    ) -> Self {
        let mut revisions = snapshots
            .iter()
            .map(|snapshot| (snapshot.path().to_path_buf(), snapshot.revision()))
            .collect::<Vec<_>>();
        revisions.sort_by(|left, right| left.0.cmp(&right.0));
        Self {
            identity: normalized_path(identity),
            revisions,
            project: project.cloned(),
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

/// Shared completed analyses, retaining one entry per project or loose source.
#[derive(Clone, Default)]
pub(super) struct AnalysisCache {
    entries: Arc<Mutex<HashMap<PathBuf, CachedAnalysis>>>,
}

struct CachedAnalysis {
    fingerprint: AnalysisFingerprint,
    analysis: Arc<AnalysisSet>,
}

impl AnalysisCache {
    /// Returns a completed analysis only when sources and project context match.
    pub(super) fn get(&self, fingerprint: &AnalysisFingerprint) -> Option<Arc<AnalysisSet>> {
        self.entries()
            .get(fingerprint.identity())
            .filter(|entry| entry.fingerprint == *fingerprint)
            .map(|entry| Arc::clone(&entry.analysis))
    }

    /// Publishes a completed analysis without holding the cache lock during computation.
    pub(super) fn insert(
        &mut self,
        fingerprint: AnalysisFingerprint,
        analysis: AnalysisSet,
    ) -> Arc<AnalysisSet> {
        let mut entries = self.entries();
        if let Some(cached) = entries
            .get(fingerprint.identity())
            .filter(|entry| entry.fingerprint == fingerprint)
        {
            return Arc::clone(&cached.analysis);
        }
        let analysis = Arc::new(analysis);
        entries.insert(
            fingerprint.identity().to_path_buf(),
            CachedAnalysis {
                fingerprint,
                analysis: Arc::clone(&analysis),
            },
        );
        analysis
    }

    /// Removes analyses owned by refreshed projects.
    pub(super) fn invalidate_identities(
        &mut self,
        identities: &std::collections::BTreeSet<PathBuf>,
    ) {
        self.entries()
            .retain(|identity, _| !identities.contains(identity));
    }

    /// Removes analyses that include a changed source or manifest.
    pub(super) fn invalidate_path(&mut self, path: &Path) {
        let path = normalized_path(path);
        self.entries().retain(|_, entry| {
            let fingerprint = &entry.fingerprint;
            fingerprint.identity() != path
                && !fingerprint
                    .revisions
                    .iter()
                    .any(|(source, _)| *source == path)
        });
    }

    fn entries(&self) -> MutexGuard<'_, HashMap<PathBuf, CachedAnalysis>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

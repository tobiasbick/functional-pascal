//! Full-text open-buffer storage with disk fallback.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{DocumentSnapshot, SourceVersion, normalized_path};
use crate::LanguageServiceError;

/// Versioned source store whose open editor buffers override disk contents.
#[derive(Clone)]
pub struct DocumentStore {
    open: HashMap<std::path::PathBuf, Arc<DocumentSnapshot>>,
    disk: Arc<Mutex<DiskSnapshots>>,
    next_snapshot_revision: Arc<AtomicU64>,
}

#[derive(Default)]
struct DiskSnapshots {
    values: HashMap<std::path::PathBuf, Arc<DocumentSnapshot>>,
    next_revision: u64,
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self {
            open: HashMap::new(),
            disk: Arc::new(Mutex::new(DiskSnapshots::default())),
            next_snapshot_revision: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl DocumentStore {
    /// Creates an empty document store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens or replaces an editor buffer with an authoritative full-text snapshot.
    pub fn open_document(
        &mut self,
        path: &Path,
        version: i64,
        source: impl Into<Arc<str>>,
    ) -> Result<Arc<DocumentSnapshot>, LanguageServiceError> {
        let path = normalized_path(path);
        if let Some(current) = self.open.get(&path)
            && let SourceVersion::Editor(current_version) = current.version()
            && version <= current_version
        {
            return Err(LanguageServiceError::StaleDocumentVersion {
                path,
                current: current_version,
                received: version,
            });
        }
        let revision = self.next_snapshot_revision(&path)?;
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Editor(version),
            revision,
            source.into(),
        ));
        self.open.insert(path, Arc::clone(&snapshot));
        Ok(snapshot)
    }

    /// Applies a newer full-text editor version to an already open document.
    pub fn apply_full_text(
        &mut self,
        path: &Path,
        version: i64,
        source: impl Into<Arc<str>>,
    ) -> Result<Arc<DocumentSnapshot>, LanguageServiceError> {
        let path = normalized_path(path);
        let Some(current) = self.open.get(&path) else {
            return Err(LanguageServiceError::DocumentNotOpen { path });
        };
        let SourceVersion::Editor(current_version) = current.version() else {
            unreachable!("open documents always have editor versions");
        };
        if version <= current_version {
            return Err(LanguageServiceError::StaleDocumentVersion {
                path,
                current: current_version,
                received: version,
            });
        }
        let revision = self.next_snapshot_revision(&path)?;
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Editor(version),
            revision,
            source.into(),
        ));
        self.open.insert(path, Arc::clone(&snapshot));
        Ok(snapshot)
    }

    /// Closes an editor buffer so subsequent loads use the current disk contents.
    pub fn close_document(&mut self, path: &Path) -> Option<Arc<DocumentSnapshot>> {
        self.open.remove(&normalized_path(path))
    }

    /// Discards a disk snapshot without affecting an authoritative open editor buffer.
    pub fn invalidate_disk(&mut self, path: &Path) {
        self.disk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values
            .remove(&normalized_path(path));
    }

    /// Returns every authoritative open editor snapshot in stable path order.
    #[must_use]
    pub fn open_snapshots(&self) -> Vec<Arc<DocumentSnapshot>> {
        let mut snapshots = self.open.values().cloned().collect::<Vec<_>>();
        snapshots.sort_by(|left, right| left.path().cmp(right.path()));
        snapshots
    }

    /// Returns the current open snapshot without reading from disk.
    #[must_use]
    pub fn open_snapshot(&self, path: &Path) -> Option<Arc<DocumentSnapshot>> {
        self.open.get(&normalized_path(path)).cloned()
    }

    /// Returns whether the path currently has an authoritative editor overlay.
    #[must_use]
    pub fn is_open(&self, path: &Path) -> bool {
        self.open.contains_key(&normalized_path(path))
    }

    /// Loads the current snapshot, preferring an open buffer and otherwise refreshing disk text.
    pub fn snapshot(&mut self, path: &Path) -> Result<Arc<DocumentSnapshot>, LanguageServiceError> {
        let path = normalized_path(path);
        if let Some(snapshot) = self.open.get(&path) {
            return Ok(Arc::clone(snapshot));
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                self.disk
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .values
                    .remove(&path);
                return Err(LanguageServiceError::source_read(&path, error));
            }
        };
        let mut disk = self
            .disk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(snapshot) = disk.values.get(&path)
            && snapshot.source() == source
        {
            return Ok(Arc::clone(snapshot));
        }

        disk.next_revision = disk.next_revision.saturating_add(1);
        let revision = self.next_snapshot_revision(&path)?;
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Disk(disk.next_revision),
            revision,
            Arc::<str>::from(source),
        ));
        disk.values.insert(path, Arc::clone(&snapshot));
        Ok(snapshot)
    }

    fn next_snapshot_revision(&self, path: &Path) -> Result<u64, LanguageServiceError> {
        self.next_snapshot_revision
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |revision| {
                revision.checked_add(1)
            })
            .map(|revision| revision + 1)
            .map_err(|_| {
                LanguageServiceError::analysis(path, "document snapshot revision space exhausted")
            })
    }
}

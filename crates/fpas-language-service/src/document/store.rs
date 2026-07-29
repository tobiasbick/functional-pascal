//! Full-text open-buffer storage with disk fallback.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::{DocumentSnapshot, SourceVersion, normalized_path};
use crate::LanguageServiceError;

/// Versioned source store whose open editor buffers override disk contents.
#[derive(Default)]
pub struct DocumentStore {
    open: HashMap<std::path::PathBuf, Arc<DocumentSnapshot>>,
    disk: HashMap<std::path::PathBuf, Arc<DocumentSnapshot>>,
    next_disk_revision: u64,
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
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Editor(version),
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
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Editor(version),
            source.into(),
        ));
        self.open.insert(path, Arc::clone(&snapshot));
        Ok(snapshot)
    }

    /// Closes an editor buffer so subsequent loads use the current disk contents.
    pub fn close_document(&mut self, path: &Path) -> Option<Arc<DocumentSnapshot>> {
        self.open.remove(&normalized_path(path))
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
                self.disk.remove(&path);
                return Err(LanguageServiceError::source_read(&path, error));
            }
        };
        if let Some(snapshot) = self.disk.get(&path)
            && snapshot.source() == source
        {
            return Ok(Arc::clone(snapshot));
        }

        self.next_disk_revision = self.next_disk_revision.saturating_add(1);
        let snapshot = Arc::new(DocumentSnapshot::parse(
            &path,
            SourceVersion::Disk(self.next_disk_revision),
            Arc::<str>::from(source),
        ));
        self.disk.insert(path, Arc::clone(&snapshot));
        Ok(snapshot)
    }
}

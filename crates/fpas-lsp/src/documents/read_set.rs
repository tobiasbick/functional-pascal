//! Delivery guards shared by source queries and edits, including unedited dependencies.

use super::{DocumentRequestError, SynchronizedDocuments};
use fpas_language_service::{LanguageService, SourceQueryError, SourceSession, SourceVersion};
use fpas_project::source_inputs::InputInventory;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct Versions {
    files: Vec<(PathBuf, Option<i64>)>,
    workspace: PathBuf,
    library: Option<PathBuf>,
}

impl Versions {
    pub(crate) fn capture(service: &LanguageService, session: &SourceSession) -> Self {
        Self {
            files: session
                .snapshot()
                .inventory
                .files
                .iter()
                .map(|file| {
                    let path = PathBuf::from(&file.path);
                    let version = open_version(service, &path);
                    (path, version)
                })
                .collect(),
            workspace: workspace(service),
            library: service.standard_library_root().map(Path::to_path_buf),
        }
    }

    pub(crate) fn matches(&self, service: &LanguageService) -> bool {
        self.workspace == workspace(service)
            && self.library.as_deref() == service.standard_library_root()
            && self
                .files
                .iter()
                .all(|(path, version)| open_version(service, path) == *version)
    }
}

impl SynchronizedDocuments {
    pub(crate) async fn verify_read_set(
        &self,
        versions: &Versions,
        inventory: InputInventory,
    ) -> Result<(), DocumentRequestError> {
        let unchanged = tokio::task::spawn_blocking(move || inventory.verify())
            .await
            .map_err(|e| DocumentRequestError::Task(e.to_string()))?
            .map_err(|e| failure("source_read", &e.to_string()))?;
        // Check editor state after the disk await, immediately before delivering the result.
        if !unchanged || !versions.matches(&*self.service.lock().await) {
            return Err(failure(
                "source_changed",
                "Participating files, buffers or workspace configuration changed before delivery.",
            ));
        }
        Ok(())
    }
}

fn workspace(service: &LanguageService) -> PathBuf {
    service
        .workspace()
        .manifest_path()
        .unwrap_or(service.workspace().root())
        .into()
}

fn open_version(service: &LanguageService, path: &Path) -> Option<i64> {
    service
        .documents()
        .open_snapshot(path)
        .and_then(|s| match s.version() {
            SourceVersion::Editor(version) => Some(version),
            SourceVersion::Disk(_) => None,
        })
}

pub(crate) fn failure(code: &'static str, message: &str) -> DocumentRequestError {
    SourceQueryError {
        code,
        message: message.into(),
        diagnostics: Vec::new(),
    }
    .into()
}

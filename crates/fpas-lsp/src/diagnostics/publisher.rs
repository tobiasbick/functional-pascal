//! Debounced, version-safe push-diagnostic publication.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_language_service::diagnostics_for_document;
use tokio::sync::Mutex;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::Uri;

use super::convert::diagnostic_to_lsp;
use crate::documents::{SynchronizedDocument, SynchronizedDocuments};

const ANALYSIS_DEBOUNCE: Duration = Duration::from_millis(120);

pub(crate) struct DiagnosticPublisher {
    client: Client,
    documents: Arc<SynchronizedDocuments>,
    generations: Arc<Mutex<GenerationState>>,
}

impl DiagnosticPublisher {
    pub(crate) fn new(client: Client, documents: Arc<SynchronizedDocuments>) -> Self {
        Self {
            client,
            documents,
            generations: Arc::new(Mutex::new(GenerationState::default())),
        }
    }

    pub(crate) async fn invalidate(&self, path: &Path) -> Option<u64> {
        self.generations.lock().await.invalidate(path)
    }

    pub(crate) fn schedule(&self, document: SynchronizedDocument, generation: u64) {
        let client = self.client.clone();
        let documents = Arc::clone(&self.documents);
        let generations = Arc::clone(&self.generations);
        tokio::spawn(async move {
            tokio::time::sleep(ANALYSIS_DEBOUNCE).await;
            if !is_current(&generations, &document.path, generation).await {
                return;
            }
            let analysis = match documents
                .analyze_if_current(&document.path, document.version)
                .await
            {
                Ok(Some(analysis)) => analysis,
                Ok(None) => return,
                Err(error) => {
                    tracing::warn!(
                        path = %document.path.display(),
                        version = document.version,
                        %error,
                        "document analysis failed"
                    );
                    return;
                }
            };
            let mut diagnostics = Vec::new();
            for diagnostic in diagnostics_for_document(&analysis) {
                match diagnostic_to_lsp(analysis.snapshot(), diagnostic) {
                    Ok(diagnostic) => diagnostics.push(diagnostic),
                    Err(error) => tracing::warn!(
                        path = %document.path.display(),
                        code = %diagnostic.code,
                        %error,
                        "cannot convert compiler diagnostic to LSP"
                    ),
                }
            }
            publish_if_current(&client, &generations, &document, generation, diagnostics).await;
        });
    }

    pub(crate) async fn cancel(&self, path: &Path) {
        self.generations.lock().await.cancel(path);
    }

    pub(crate) async fn cancel_and_clear(&self, path: &Path, uri: Uri) {
        let mut generations = self.generations.lock().await;
        generations.cancel(path);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    pub(crate) async fn shutdown(&self) {
        self.generations.lock().await.shutdown();
    }
}

async fn is_current(generations: &Mutex<GenerationState>, path: &Path, generation: u64) -> bool {
    generations.lock().await.is_current(path, generation)
}

async fn publish_if_current(
    client: &Client,
    generations: &Mutex<GenerationState>,
    document: &SynchronizedDocument,
    generation: u64,
    diagnostics: Vec<tower_lsp_server::ls_types::Diagnostic>,
) {
    let generations = generations.lock().await;
    if generations.is_current(&document.path, generation) {
        client
            .publish_diagnostics(document.uri.clone(), diagnostics, Some(document.version))
            .await;
    }
}

#[derive(Default)]
struct GenerationState {
    next: u64,
    current: HashMap<PathBuf, u64>,
    stopped: bool,
}

impl GenerationState {
    fn invalidate(&mut self, path: &Path) -> Option<u64> {
        if self.stopped {
            return None;
        }
        self.next = self.next.wrapping_add(1);
        let generation = self.next;
        self.current.insert(path.to_path_buf(), generation);
        Some(generation)
    }

    fn cancel(&mut self, path: &Path) {
        self.current.remove(path);
    }

    fn is_current(&self, path: &Path, generation: u64) -> bool {
        !self.stopped && self.current.get(path) == Some(&generation)
    }

    fn shutdown(&mut self) {
        self.stopped = true;
        self.current.clear();
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "hard-coded generation fixtures use expect to keep failures local"
)]
mod tests {
    use std::path::Path;

    use super::GenerationState;

    #[test]
    fn newer_generation_and_cancel_invalidate_older_work() {
        let path = Path::new("versioned.fpas");
        let mut state = GenerationState::default();
        let first = state.invalidate(path).expect("first generation");
        let second = state.invalidate(path).expect("second generation");

        assert!(!state.is_current(path, first));
        assert!(state.is_current(path, second));
        state.cancel(path);
        assert!(!state.is_current(path, second));
    }

    #[test]
    fn shutdown_rejects_every_later_generation() {
        let path = Path::new("shutdown.fpas");
        let mut state = GenerationState::default();
        let generation = state.invalidate(path).expect("generation");

        state.shutdown();

        assert!(!state.is_current(path, generation));
        assert_eq!(state.invalidate(path), None);
    }
}

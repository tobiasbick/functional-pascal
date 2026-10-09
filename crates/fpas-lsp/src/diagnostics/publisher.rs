//! Debounced, version-safe push-diagnostic publication.
//!
//! **Documentation:** `docs/pascal/tools/editor-integration.md`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fpas_language_service::diagnostics_for_document;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore, mpsc};
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::Uri;

use super::convert::diagnostic_to_lsp;
use super::project::project_diagnostics;
use super::publication::{DiagnosticBatch, Publication, dispatch_publications};
use crate::documents::{SynchronizedDocument, SynchronizedDocuments};

const ANALYSIS_DEBOUNCE: Duration = Duration::from_millis(120);

/// Coordinates debounced analysis with ordered, version-safe diagnostic delivery.
pub(crate) struct DiagnosticPublisher {
    documents: Arc<SynchronizedDocuments>,
    generations: Arc<Mutex<GenerationState>>,
    analysis_slots: Arc<Semaphore>,
    publications: mpsc::UnboundedSender<Publication>,
}

impl DiagnosticPublisher {
    /// Starts the publication lane for this editor session.
    pub(crate) fn new(client: Client, documents: Arc<SynchronizedDocuments>) -> Self {
        let generations = Arc::new(Mutex::new(GenerationState::default()));
        let (publications, receiver) = mpsc::unbounded_channel();
        tokio::spawn(dispatch_publications(
            client,
            Arc::clone(&generations),
            Arc::clone(&documents),
            receiver,
        ));
        Self {
            documents,
            generations,
            analysis_slots: Arc::new(Semaphore::new(1)),
            publications,
        }
    }

    pub(crate) async fn invalidate(&self, path: &Path) -> Option<u64> {
        self.generations.lock().await.invalidate(path)
    }

    /// Schedules current-buffer analysis and preserves project failure attribution.
    pub(crate) fn schedule(&self, document: SynchronizedDocument, generation: u64) {
        let documents = Arc::clone(&self.documents);
        let generations = Arc::clone(&self.generations);
        let analysis_slots = Arc::clone(&self.analysis_slots);
        let publications = self.publications.clone();
        tokio::spawn(async move {
            tokio::time::sleep(ANALYSIS_DEBOUNCE).await;
            let Some(_analysis_permit) = acquire_current_analysis_slot(
                &generations,
                analysis_slots,
                &document.path,
                generation,
            )
            .await
            else {
                return;
            };
            let analysis = match documents
                .analyze_diagnostics_if_current(&document.path, document.version)
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
            for diagnostic in diagnostics_for_document(analysis.document()) {
                match diagnostic_to_lsp(analysis.document().snapshot(), diagnostic) {
                    Ok(diagnostic) => diagnostics.push(diagnostic),
                    Err(error) => tracing::warn!(
                        path = %document.path.display(),
                        code = %diagnostic.code,
                        %error,
                        "cannot convert compiler diagnostic to LSP"
                    ),
                }
            }
            let (mut batches, unlocated) = project_diagnostics(&analysis);
            batches.push(DiagnosticBatch {
                uri: document.uri.clone(),
                version: Some(document.version),
                revision: analysis.document().snapshot().revision(),
                diagnostics,
            });
            let _ = publications.send(Publication::Diagnostics {
                document,
                generation,
                batches,
                unlocated,
            });
        });
    }

    pub(crate) async fn cancel(&self, path: &Path) {
        self.generations.lock().await.cancel(path);
    }

    pub(crate) async fn cancel_and_clear(&self, path: &Path, uri: Uri) {
        self.generations.lock().await.cancel(path);
        let _ = self.publications.send(Publication::Clear { uri });
    }

    pub(crate) async fn shutdown(&self) {
        self.generations.lock().await.shutdown();
    }
}

/// Returns whether an origin's analysis generation is still publishable.
pub(super) async fn is_current(
    generations: &Mutex<GenerationState>,
    path: &Path,
    generation: u64,
) -> bool {
    generations.lock().await.is_current(path, generation)
}

async fn acquire_current_analysis_slot(
    generations: &Mutex<GenerationState>,
    analysis_slots: Arc<Semaphore>,
    path: &Path,
    generation: u64,
) -> Option<OwnedSemaphorePermit> {
    if !is_current(generations, path, generation).await {
        return None;
    }
    let permit = analysis_slots.acquire_owned().await.ok()?;
    is_current(generations, path, generation)
        .await
        .then_some(permit)
}

/// Current origin generations and shutdown state.
#[derive(Default)]
pub(super) struct GenerationState {
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
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use tokio::sync::{Mutex, Semaphore};
    use tower_lsp_server::ls_types::{InitializeParams, InitializeResult, Uri};
    use tower_lsp_server::{Client, LanguageServer, LspService};

    use super::{DiagnosticPublisher, GenerationState, acquire_current_analysis_slot};
    use crate::documents::SynchronizedDocuments;

    struct TestServer {
        client: Client,
    }

    impl LanguageServer for TestServer {
        async fn initialize(
            &self,
            _params: InitializeParams,
        ) -> tower_lsp_server::jsonrpc::Result<InitializeResult> {
            Ok(InitializeResult::default())
        }

        async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
            Ok(())
        }
    }

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

    #[tokio::test]
    async fn queued_analysis_rechecks_generation_after_acquiring_slot() {
        let path = PathBuf::from("versioned.fpas");
        let generations = Arc::new(Mutex::new(GenerationState::default()));
        let generation = generations
            .lock()
            .await
            .invalidate(&path)
            .expect("generation");
        let analysis_slots = Arc::new(Semaphore::new(1));
        let occupied_slot = Arc::clone(&analysis_slots)
            .acquire_owned()
            .await
            .expect("analysis slot");
        let queued_generations = Arc::clone(&generations);
        let queued_slots = Arc::clone(&analysis_slots);
        let queued_path = path.clone();
        let queued = tokio::spawn(async move {
            acquire_current_analysis_slot(
                &queued_generations,
                queued_slots,
                &queued_path,
                generation,
            )
            .await
        });
        tokio::task::yield_now().await;

        generations.lock().await.invalidate(&path);
        drop(occupied_slot);

        assert!(queued.await.expect("queued analysis completes").is_none());
    }

    #[tokio::test]
    async fn backpressured_client_does_not_hold_generation_state() {
        let (service, _socket) = LspService::new(|client| TestServer { client });
        let client = service.inner().client.clone();
        let uri = "file:///backpressure.fpas".parse::<Uri>().expect("URI");
        client
            .publish_diagnostics(uri.clone(), Vec::new(), None)
            .await;
        let publisher =
            DiagnosticPublisher::new(client, Arc::new(SynchronizedDocuments::new(PathBuf::new())));
        publisher
            .cancel_and_clear(Path::new("backpressure.fpas"), uri)
            .await;
        tokio::task::yield_now().await;

        let generation = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            publisher.invalidate(Path::new("other.fpas")),
        )
        .await
        .expect("generation state remains responsive");
        assert!(generation.is_some());
        tokio::time::timeout(std::time::Duration::from_millis(100), publisher.shutdown())
            .await
            .expect("shutdown remains responsive");
    }
}

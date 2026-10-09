//! Ordered diagnostic delivery with source attribution and shared-target clearing.
//!
//! **Documentation:** `docs/pascal/tools/editor-integration.md`

use std::collections::BTreeMap;
use std::sync::Arc;

use fpas_diagnostics::{DiagnosticSeverity, FileDiagnostic};
use fpas_language_service::SourceVersion;
use tokio::sync::{Mutex, mpsc};
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::{Diagnostic, MessageType, Uri};

use super::publisher::{GenerationState, is_current};
use crate::convert::file_uri_to_path;
use crate::documents::{SynchronizedDocument, SynchronizedDocuments};

/// Located records for one authoritative source revision.
pub(super) struct DiagnosticBatch {
    /// Actual source document receiving the markers.
    pub(super) uri: Uri,
    /// Editor version, or absent for a closed disk source.
    pub(super) version: Option<i32>,
    /// Source revision distinguishes separate editor lifetimes reusing one version.
    pub(super) revision: u64,
    /// Records converted using that source's snapshot.
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// One generation-checked diagnostic update or source-close operation.
pub(super) enum Publication {
    /// Replace an origin document's own and related project records.
    Diagnostics {
        /// Open document whose analysis produced these records.
        document: SynchronizedDocument,
        /// Current analysis generation for the origin.
        generation: u64,
        /// Own and related source publications.
        batches: Vec<DiagnosticBatch>,
        /// Coded failures that have no usable source range.
        unlocated: Vec<FileDiagnostic>,
    },
    /// Remove the records produced by an origin document.
    Clear {
        /// Closed origin document.
        uri: Uri,
    },
}

/// Deliver updates without holding generation or document locks during client backpressure.
pub(super) async fn dispatch_publications(
    client: Client,
    generations: Arc<Mutex<GenerationState>>,
    documents: Arc<SynchronizedDocuments>,
    mut receiver: mpsc::UnboundedReceiver<Publication>,
) {
    let mut origins: BTreeMap<String, Vec<DiagnosticBatch>> = BTreeMap::new();
    while let Some(publication) = receiver.recv().await {
        let (origin, batches, unlocated) = match publication {
            Publication::Diagnostics {
                document,
                generation,
                batches,
                unlocated,
            } => {
                if !is_current(&generations, &document.path, generation).await {
                    continue;
                }
                (document.uri, Some(batches), unlocated)
            }
            Publication::Clear { uri } => (uri, None, Vec::new()),
        };
        let closing = batches.is_none();
        let mut targets = BTreeMap::new();
        targets.insert(origin.to_string(), origin.clone());
        if let Some(previous) = origins.remove(&origin.to_string()) {
            for batch in previous {
                targets.insert(batch.uri.to_string(), batch.uri);
            }
        }
        if let Some(batches) = batches {
            for batch in &batches {
                targets.insert(batch.uri.to_string(), batch.uri.clone());
            }
            origins.insert(origin.to_string(), batches);
        }
        for uri in targets.into_values() {
            let Ok(path) = file_uri_to_path(&uri) else {
                continue;
            };
            let snapshot = documents
                .service
                .lock()
                .await
                .documents()
                .open_snapshot(&path);
            let version = snapshot
                .as_ref()
                .and_then(|snapshot| match snapshot.version() {
                    SourceVersion::Editor(version) => i32::try_from(version).ok(),
                    SourceVersion::Disk(_) => None,
                });
            let mut diagnostics = Vec::new();
            for batch in origins.values().flatten().filter(|batch| {
                batch.uri == uri
                    && batch.version == version
                    && snapshot
                        .as_ref()
                        .is_none_or(|snapshot| snapshot.revision() == batch.revision)
            }) {
                for diagnostic in &batch.diagnostics {
                    if !diagnostics.contains(diagnostic) {
                        diagnostics.push(diagnostic.clone());
                    }
                }
            }
            let publication_version = if closing && uri == origin {
                None
            } else {
                version
            };
            client
                .publish_diagnostics(uri, diagnostics, publication_version)
                .await;
        }
        for record in unlocated {
            let severity = match record.diagnostic.severity {
                DiagnosticSeverity::Error => MessageType::ERROR,
                DiagnosticSeverity::Warning => MessageType::WARNING,
            };
            client.log_message(severity, record.to_string()).await;
        }
    }
}

//! Blocking, cooperatively cancellable navigation request tasks.

use std::path::PathBuf;
use std::sync::Arc;

use fpas_language_service::{CancellationToken, LanguageService};
use tokio::sync::Mutex;
use tower_lsp_server::ls_types::Position;

use crate::convert::position_to_byte_offset;
use crate::documents::{DocumentRequestError, ReferenceDocument, RenameDocument, require_open};

struct CancelOnDrop(CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

pub(crate) async fn references(
    service: Arc<Mutex<LanguageService>>,
    path: PathBuf,
    position: Position,
    include_declaration: bool,
) -> Result<Vec<ReferenceDocument>, DocumentRequestError> {
    let cancellation = CancellationToken::new();
    let task_cancellation = cancellation.clone();
    let _cancel_on_drop = CancelOnDrop(cancellation);
    tokio::task::spawn_blocking(move || {
        let mut service = service.blocking_lock();
        let snapshot = require_open(&service, &path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        let result = service.references_with_cancellation(
            &path,
            offset,
            include_declaration,
            &task_cancellation,
        )?;
        let mut references = Vec::with_capacity(result.value.len());
        for location in result.value {
            task_cancellation
                .check()
                .map_err(DocumentRequestError::from)?;
            references.push(ReferenceDocument {
                snapshot: service.snapshot(&location.path)?,
                location,
            });
        }
        Ok(references)
    })
    .await
    .map_err(|error| DocumentRequestError::Task(error.to_string()))?
}

pub(crate) async fn rename(
    service: Arc<Mutex<LanguageService>>,
    path: PathBuf,
    position: Position,
    new_name: String,
) -> Result<Vec<RenameDocument>, DocumentRequestError> {
    let cancellation = CancellationToken::new();
    let task_cancellation = cancellation.clone();
    let _cancel_on_drop = CancelOnDrop(cancellation);
    tokio::task::spawn_blocking(move || {
        let mut service = service.blocking_lock();
        let snapshot = require_open(&service, &path)?;
        let offset = position_to_byte_offset(&snapshot, position)?;
        let result =
            service.rename_with_cancellation(&path, offset, &new_name, &task_cancellation)?;
        let mut edits = Vec::with_capacity(result.value.len());
        for edit in result.value {
            task_cancellation
                .check()
                .map_err(DocumentRequestError::from)?;
            edits.push(RenameDocument {
                snapshot: service.snapshot(&edit.path)?,
                edit,
            });
        }
        Ok(edits)
    })
    .await
    .map_err(|error| DocumentRequestError::Task(error.to_string()))?
}

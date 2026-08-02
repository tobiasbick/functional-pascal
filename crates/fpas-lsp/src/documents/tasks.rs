//! Blocking query execution over isolated immutable document snapshots.

use std::sync::Arc;

use fpas_language_service::{CancellationToken, LanguageService};
use tokio::sync::Mutex;

use super::DocumentRequestError;

pub(super) struct CancelOnDrop(pub(super) CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

pub(crate) async fn run<T, F>(
    service: &Arc<Mutex<LanguageService>>,
    query: F,
) -> Result<T, DocumentRequestError>
where
    T: Send + 'static,
    F: FnOnce(&mut LanguageService, &CancellationToken) -> Result<T, DocumentRequestError>
        + Send
        + 'static,
{
    let mut query_service = service.lock().await.fork_for_queries();
    let cancellation = CancellationToken::new();
    let task_cancellation = cancellation.clone();
    let _cancel_on_drop = CancelOnDrop(cancellation);
    tokio::task::spawn_blocking(move || query(&mut query_service, &task_cancellation))
        .await
        .map_err(|error| DocumentRequestError::Task(error.to_string()))?
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "test synchronization failures need local context"
)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fpas_language_service::{LanguageService, LanguageServiceError};
    use tokio::sync::{Mutex, oneshot};

    use super::run;

    #[tokio::test]
    async fn query_releases_primary_service_before_blocking_work() {
        let service = Arc::new(Mutex::new(LanguageService::load(&PathBuf::new())));
        let (started_tx, started_rx) = oneshot::channel();
        let (finish_tx, finish_rx) = oneshot::channel();
        let query_service = Arc::clone(&service);
        let query = tokio::spawn(async move {
            run(&query_service, move |_service, _cancellation| {
                started_tx.send(()).expect("report query start");
                finish_rx.blocking_recv().expect("release query");
                Ok(())
            })
            .await
        });

        started_rx.await.expect("query started");
        assert!(service.try_lock().is_ok());
        finish_tx.send(()).expect("finish query");
        query.await.expect("join query").expect("query succeeds");
    }

    #[tokio::test]
    async fn dropping_query_future_cancels_blocking_work() {
        let service = Arc::new(Mutex::new(LanguageService::load(&PathBuf::new())));
        let (started_tx, started_rx) = oneshot::channel();
        let (cancelled_tx, cancelled_rx) = oneshot::channel();
        let query = tokio::spawn(async move {
            run(&service, move |_service, cancellation| {
                started_tx.send(()).expect("report query start");
                while !cancellation.is_cancelled() {
                    std::thread::yield_now();
                }
                cancelled_tx.send(()).expect("report cancellation");
                Err::<(), _>(LanguageServiceError::Cancelled.into())
            })
            .await
        });

        started_rx.await.expect("query started");
        query.abort();
        cancelled_rx
            .await
            .expect("blocking query observed cancellation");
    }
}

//! Executor construction and queued opening retirement run on the owned native thread.

use super::*;

pub(super) fn run(
    build: impl FnOnce() -> Result<tokio::runtime::Runtime, HttpError>,
    state: Arc<Mutex<State>>,
    shutdown: CancellationToken,
    mut receiver: tokio::sync::mpsc::Receiver<Open>,
) {
    let runtime = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build))
        .unwrap_or(Err(HttpError::Executor))
    {
        Ok(runtime) => runtime,
        Err(error) => {
            state.lock().unwrap_or_else(|p| p.into_inner()).failure = Some(error.clone());
            receiver.close();
            while let Ok(open) = receiver.try_recv() {
                open.request.finish(Err(error.clone()));
                open.receipt.finish(Ok(()));
            }
            return;
        }
    };
    runtime.block_on(async move {
        let mut tasks = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => break,
                Some(_) = tasks.join_next(), if !tasks.is_empty() => {},
                open = receiver.recv() => {
                    let Some(open) = open else { break; };
                    tasks.spawn(open_publication(open, shutdown.clone()));
                },
            }
        }
        receiver.close();
        while let Some(open) = receiver.recv().await {
            open.request.finish(Err(Fault::Closed.into()));
            open.receipt.finish(Ok(()));
        }
        while tasks.join_next().await.is_some() {}
    });
}

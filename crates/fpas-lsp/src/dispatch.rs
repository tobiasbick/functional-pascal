//! Dispatch-edge ordering for document mutations and dependent requests.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

use tokio::sync::watch;
use tower::Service;
use tower_lsp_server::jsonrpc::{Request, Response};
use tower_lsp_server::{ExitedError, LspService};

use crate::Backend;

/// LSP service wrapper that preserves input order for document mutations.
///
/// Requests remain concurrent, but each waits for document notifications dispatched before it.
pub struct OrderedLspService {
    inner: LspService<Backend>,
    order: Arc<DocumentOrder>,
}

impl OrderedLspService {
    pub(crate) fn new(inner: LspService<Backend>) -> Self {
        Self {
            inner,
            order: Arc::new(DocumentOrder::new()),
        }
    }
}

impl Service<Request> for OrderedLspService {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let method = request.method().to_owned();
        let mutation = is_document_mutation(&method);
        let required = if mutation {
            self.order.next_sequence()
        } else {
            self.order.last_sequence()
        };
        let previous = required.saturating_sub(u64::from(mutation));
        let order = Arc::clone(&self.order);
        let future = self.inner.call(request);
        Box::pin(async move {
            if method != "$/cancelRequest" {
                order.wait_for(previous).await;
            }
            let result = future.await;
            if mutation {
                order.complete(required);
            }
            result
        })
    }
}

struct DocumentOrder {
    next: AtomicU64,
    completed: watch::Sender<u64>,
}

impl DocumentOrder {
    fn new() -> Self {
        let (completed, _) = watch::channel(0);
        Self {
            next: AtomicU64::new(0),
            completed,
        }
    }

    fn next_sequence(&self) -> u64 {
        self.next.fetch_add(1, Ordering::AcqRel).saturating_add(1)
    }

    fn last_sequence(&self) -> u64 {
        self.next.load(Ordering::Acquire)
    }

    async fn wait_for(&self, required: u64) {
        let mut completed = self.completed.subscribe();
        while *completed.borrow_and_update() < required {
            if completed.changed().await.is_err() {
                break;
            }
        }
    }

    fn complete(&self, sequence: u64) {
        self.completed.send_replace(sequence);
    }
}

fn is_document_mutation(method: &str) -> bool {
    matches!(
        method,
        "textDocument/didOpen"
            | "textDocument/didChange"
            | "textDocument/didSave"
            | "textDocument/didClose"
            | "workspace/didChangeWatchedFiles"
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{DocumentOrder, is_document_mutation};

    #[test]
    fn document_mutation_methods_are_classified_at_dispatch() {
        assert!(is_document_mutation("textDocument/didOpen"));
        assert!(is_document_mutation("textDocument/didChange"));
        assert!(is_document_mutation("textDocument/didClose"));
        assert!(is_document_mutation("workspace/didChangeWatchedFiles"));
        assert!(!is_document_mutation("textDocument/completion"));
    }

    #[tokio::test]
    async fn requests_wait_for_every_preceding_mutation_sequence() {
        let order = Arc::new(DocumentOrder::new());
        let first = order.next_sequence();
        let second = order.next_sequence();
        assert_eq!((first, second, order.last_sequence()), (1, 2, 2));

        order.complete(first);
        let waiting_order = Arc::clone(&order);
        let waiting = tokio::spawn(async move { waiting_order.wait_for(second).await });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());

        order.complete(second);
        assert!(waiting.await.is_ok());
    }
}

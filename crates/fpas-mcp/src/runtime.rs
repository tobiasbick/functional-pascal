//! Native-only executor, cancellable openings and joined transport receipts.

use crate::{Endpoint, Fault, HttpEndpoint, HttpError, HttpOptions, Server};
use std::sync::{Arc, Mutex, Weak};
use std::thread::JoinHandle;
use tokio_util::sync::CancellationToken;

const OPEN_QUEUE: usize = 16;
const PUBLICATIONS: usize = 256;
mod startup;
#[cfg(test)]
mod tests;
type Cleanup = Result<(), HttpError>;
type OpenResult = Result<Arc<Publication>, HttpError>;

struct Completion {
    result: Mutex<Option<Cleanup>>,
    changed: tokio::sync::Notify,
}

/// A native-only receipt available before a listener starts opening.
#[derive(Clone)]
pub struct CleanupReceipt(Arc<Completion>);
impl CleanupReceipt {
    fn new() -> Self {
        Self(Arc::new(Completion {
            result: Mutex::new(None),
            changed: tokio::sync::Notify::new(),
        }))
    }
    fn finish(&self, result: Cleanup) {
        let mut stored = self.0.result.lock().unwrap_or_else(|e| e.into_inner());
        if stored.is_none() {
            *stored = Some(result);
            self.0.changed.notify_waiters();
        }
    }
    /// Observe a terminal transport outcome without blocking the VM.
    pub fn result(&self) -> Option<Cleanup> {
        self.0
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    /// Await joined physical cleanup on a native executor.
    pub async fn wait(&self) -> Cleanup {
        loop {
            let changed = self.0.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(result) = self.result() {
                return result;
            }
            changed.await;
        }
    }
}

struct Record {
    server: Weak<()>,
    receipt: Weak<Completion>,
}
struct State {
    stopping: bool,
    failure: Option<HttpError>,
    opening: usize,
    publications: Vec<Weak<Publication>>,
    records: Vec<Record>,
}
struct RequestState {
    cancelled: bool,
    completed: bool,
    result: Option<OpenResult>,
}
struct Request {
    state: Mutex<RequestState>,
    changed: tokio::sync::Notify,
}
impl Request {
    fn finish(&self, result: OpenResult) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.cancelled {
            if let Ok(publication) = result {
                publication.stop();
            }
        } else if !state.completed && state.result.is_none() {
            state.result = Some(result);
            self.changed.notify_waiters();
        }
    }
}
struct Open {
    server: Server,
    options: HttpOptions,
    connection: Option<u64>,
    request: Arc<Request>,
    receipt: CleanupReceipt,
    reservation: Reservation,
}
struct Reservation {
    state: Arc<Mutex<State>>,
    active: bool,
}
impl Reservation {
    fn publish(&mut self, publication: &Arc<Publication>) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.opening -= 1;
        state.publications.push(Arc::downgrade(publication));
        self.active = false;
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.active {
            self.state.lock().unwrap_or_else(|e| e.into_inner()).opening -= 1;
        }
    }
}
struct OperationGuard {
    request: Arc<Request>,
    receipt: CleanupReceipt,
    authority: Option<Endpoint>,
}
impl Drop for OperationGuard {
    fn drop(&mut self) {
        if let Some(authority) = &self.authority {
            authority.close();
        }
        self.receipt.finish(Err(HttpError::Executor));
        self.request.finish(Err(HttpError::Executor));
    }
}

/// Independently authenticated access, authority and cleanup; never formats credentials.
pub struct Publication {
    authority: Endpoint,
    url: String,
    authorization: String,
    cleanup: CleanupReceipt,
}
impl Publication {
    /// Server-local attachment identity.
    pub fn id(&self) -> u64 {
        self.authority.id()
    }
    /// MCP URL without credentials.
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Confidential Authorization header.
    pub fn authorization(&self) -> &str {
        &self.authorization
    }
    /// Immediately retire access and response rights.
    pub fn stop(&self) -> bool {
        self.authority.close()
    }
    /// Bind a prepared attachment once, after the owner confirms session setup.
    pub fn bind_session(&self, session: &str) -> Result<(), Fault> {
        self.authority.bind_session(session)
    }
    /// Joined cleanup outcome, when available.
    pub fn cleanup_result(&self) -> Option<Cleanup> {
        self.cleanup.result()
    }
    /// Retain cleanup independently of the access handle.
    pub fn receipt(&self) -> CleanupReceipt {
        self.cleanup.clone()
    }
}

/// Exclusive opening ownership; dropping an unconsumed ticket revokes its publication.
pub struct PublishTicket {
    request: Arc<Request>,
    receipt: CleanupReceipt,
}
impl PublishTicket {
    /// Retain cleanup before abandoning an opening or consuming its access result.
    pub fn cleanup_receipt(&self) -> CleanupReceipt {
        self.receipt.clone()
    }
    /// Consume one completed opening result; later attempts report Closed.
    pub fn try_complete(&mut self) -> Option<OpenResult> {
        let mut state = self.request.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.completed {
            return Some(Err(Fault::Closed.into()));
        }
        let result = state.result.take()?;
        state.completed = true;
        Some(result)
    }
    /// Await the opening on a native executor; FPAS uses cooperative polling.
    pub async fn complete(&mut self) -> OpenResult {
        let request = self.request.clone();
        loop {
            let changed = request.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(result) = self.try_complete() {
                return result;
            }
            changed.await;
        }
    }
}
impl Drop for PublishTicket {
    fn drop(&mut self) {
        let mut state = self.request.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.completed {
            return;
        }
        state.cancelled = true;
        if let Some(Ok(publication)) = state.result.take() {
            publication.stop();
        }
    }
}

/// One bounded native executor per VM, containing no FPAS values or callback execution.
pub struct HttpRuntime {
    state: Arc<Mutex<State>>,
    opens: tokio::sync::mpsc::Sender<Open>,
    shutdown: CancellationToken,
    thread: Mutex<Option<JoinHandle<()>>>,
}
impl HttpRuntime {
    /// Start owned executor initialization without waiting or binding listeners.
    /// Initialization failures complete queued publication tickets with their cause.
    pub fn new() -> Result<Self, HttpError> {
        Self::with_builder(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(HttpError::from)
        })
    }
    fn with_builder(
        build: impl FnOnce() -> Result<tokio::runtime::Runtime, HttpError> + Send + 'static,
    ) -> Result<Self, HttpError> {
        let state = Arc::new(Mutex::new(State {
            stopping: false,
            failure: None,
            opening: 0,
            publications: Vec::new(),
            records: Vec::new(),
        }));
        let shutdown = CancellationToken::new();
        let owned_shutdown = shutdown.clone();
        let (opens, receiver) = tokio::sync::mpsc::channel::<Open>(OPEN_QUEUE);
        let owned_state = state.clone();
        let thread = std::thread::Builder::new()
            .name("fpas-mcp".into())
            .spawn(move || {
                startup::run(build, owned_state, owned_shutdown, receiver);
            })
            .map_err(HttpError::from)?;
        Ok(Self {
            state,
            opens,
            shutdown,
            thread: Mutex::new(Some(thread)),
        })
    }
    /// Reserve bounded opening capacity before physical publication.
    pub fn publish(
        &self,
        server: Server,
        options: HttpOptions,
    ) -> Result<PublishTicket, HttpError> {
        self.open(server, options, None)
    }
    /// Prepare session authority before revealing access to an agent.
    pub fn publish_attached(
        &self,
        server: Server,
        options: HttpOptions,
        connection: u64,
    ) -> Result<PublishTicket, HttpError> {
        self.open(server, options, Some(connection))
    }
    fn open(
        &self,
        server: Server,
        options: HttpOptions,
        connection: Option<u64>,
    ) -> Result<PublishTicket, HttpError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.stopping || server.is_closed() {
            return Err(Fault::Closed.into());
        }
        if let Some(error) = &state.failure {
            return Err(error.clone());
        }
        state
            .publications
            .retain(|weak| weak.upgrade().is_some_and(|p| p.cleanup_result().is_none()));
        state.records.retain(|record| {
            record.receipt.upgrade().is_some_and(|receipt| {
                receipt
                    .result
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .is_none()
            })
        });
        if state.publications.len() + state.opening >= PUBLICATIONS {
            return Err(Fault::LimitExceeded("Publications", PUBLICATIONS).into());
        }
        let permit = self
            .opens
            .try_reserve()
            .map_err(|_| HttpError::Server(Fault::Busy))?;
        let request = Arc::new(Request {
            state: Mutex::new(RequestState {
                cancelled: false,
                completed: false,
                result: None,
            }),
            changed: tokio::sync::Notify::new(),
        });
        let receipt = CleanupReceipt::new();
        state.records.push(Record {
            server: server.identity(),
            receipt: Arc::downgrade(&receipt.0),
        });
        state.opening += 1;
        permit.send(Open {
            server,
            options,
            connection,
            request: request.clone(),
            receipt: receipt.clone(),
            reservation: Reservation {
                state: self.state.clone(),
                active: true,
            },
        });
        Ok(PublishTicket { request, receipt })
    }
    /// Atomically stop a server against new openings and retain every cleanup receipt.
    pub fn stop_server(&self, server: &Server) -> (bool, Vec<CleanupReceipt>) {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let identity = server.identity();
        let receipts = state
            .records
            .iter()
            .filter(|record| Weak::ptr_eq(&record.server, &identity))
            .filter_map(|record| record.receipt.upgrade())
            .map(CleanupReceipt)
            .collect();
        (server.close(), receipts)
    }
    /// Stop admission and revoke every published authority.
    pub fn request_shutdown(&self) {
        let publications = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.stopping = true;
            state
                .publications
                .iter()
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>()
        };
        for publication in publications {
            publication.stop();
        }
        self.shutdown.cancel();
    }
    /// Join only native tasks, after VM workers stop polling.
    pub fn wait_closed(&self) {
        if let Some(thread) = self.thread.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = thread.join();
        }
    }
}
impl Drop for HttpRuntime {
    fn drop(&mut self) {
        self.request_shutdown();
        self.wait_closed();
    }
}
async fn open_publication(mut open: Open, shutdown: CancellationToken) {
    let mut guard = OperationGuard {
        request: open.request.clone(),
        receipt: open.receipt.clone(),
        authority: None,
    };
    let mut endpoint = match HttpEndpoint::bind(&open.server, open.options).await {
        Ok(endpoint) => endpoint,
        Err(error) => {
            open.request.finish(Err(error));
            open.receipt.finish(Ok(()));
            return;
        }
    };
    guard.authority = Some(endpoint.authority());
    if let Some(connection) = open.connection
        && let Err(error) = endpoint.authority().begin_attachment(connection)
    {
        let cleanup = endpoint.close().await;
        open.receipt.finish(cleanup.clone());
        open.request
            .finish(Err(cleanup.err().unwrap_or_else(|| error.into())));
        return;
    }
    let publication = Arc::new(Publication {
        authority: endpoint.authority(),
        url: endpoint.url(),
        authorization: endpoint.authorization().to_owned(),
        cleanup: open.receipt.clone(),
    });
    open.reservation.publish(&publication);
    open.request.finish(Ok(publication.clone()));
    tokio::select! {
        _ = publication.authority.cancelled() => {},
        _ = shutdown.cancelled() => { publication.stop(); },
    }
    open.receipt.finish(endpoint.close().await);
}

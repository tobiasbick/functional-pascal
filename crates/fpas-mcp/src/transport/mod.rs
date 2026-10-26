//! Authenticated loopback Streamable HTTP attachments with owned socket tasks.

mod error;
mod http;
mod options;

pub use error::HttpError;
pub use options::HttpOptions;

use crate::{Endpoint, Server};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_util::sync::CancellationToken;

/// One independently authenticated publication of a caller-owned server.
///
/// This value intentionally implements no Debug: its authorization header is
/// confidential. Closing it retires its endpoint and preserves the server.
pub struct HttpEndpoint {
    endpoint: Endpoint,
    address: SocketAddr,
    authorization: Arc<str>,
    cancellation: CancellationToken,
    task: Option<JoinHandle<Result<(), HttpError>>>,
}

impl HttpEndpoint {
    /// Clone authority for a runtime that separately owns physical cleanup.
    pub fn authority(&self) -> Endpoint {
        self.endpoint.clone()
    }
    /// Bind a loopback listener and publish the frozen server catalog.
    ///
    /// Call from the embedding application's Tokio executor. Each publication
    /// receives independent credentials, endpoint identity and socket limits.
    pub async fn bind(server: &Server, mut options: HttpOptions) -> Result<Self, HttpError> {
        options.validate()?;
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|_| HttpError::Entropy)?;
        let mut authorization = String::from("Bearer ");
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in random {
            authorization.push(HEX[(byte >> 4) as usize] as char);
            authorization.push(HEX[(byte & 15) as usize] as char);
        }
        let authorization: Arc<str> = authorization.into();
        let listener = TcpListener::bind(options.address).await?;
        let address = listener.local_addr()?;
        options.address = address;
        let endpoint = server.endpoint()?;
        let cancellation = CancellationToken::new();
        let task = tokio::spawn(http::run(
            listener,
            endpoint.clone(),
            authorization.clone(),
            cancellation.clone(),
            options,
            server.limits().message_bytes,
        ));
        Ok(Self {
            endpoint,
            address,
            authorization,
            cancellation,
            task: Some(task),
        })
    }

    /// Server-local identity carried by every application call on this route.
    pub fn id(&self) -> u64 {
        self.endpoint.id()
    }

    /// Actual bound loopback address, including its operating-system-assigned port.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// MCP URL without access credentials.
    pub fn url(&self) -> String {
        format!("http://{}/mcp", self.address)
    }

    /// Confidential Authorization header for an explicitly authorized consumer.
    pub fn authorization(&self) -> &str {
        &self.authorization
    }

    /// Retire access and response rights immediately; socket cleanup is asynchronous.
    pub fn stop(&self) {
        self.endpoint.close();
        self.cancellation.cancel();
    }

    /// Retire the attachment and join owned sockets within the shutdown deadline.
    ///
    /// Repeated closure succeeds. The underlying server and other routes survive.
    pub async fn close(&mut self) -> Result<(), HttpError> {
        self.stop();
        let Some(task) = self.task.as_mut() else {
            return Ok(());
        };
        let result = task.await.map_err(|_| HttpError::Executor);
        self.task = None;
        result?
    }
}

impl Drop for HttpEndpoint {
    fn drop(&mut self) {
        self.stop();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

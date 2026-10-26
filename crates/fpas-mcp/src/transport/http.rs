//! Owned HTTP sockets around the SDK's stateless Streamable HTTP service.

use super::{HttpError, HttpOptions};
use crate::Endpoint;
use bytes::Bytes;
use http::{Request, Response, StatusCode, header::AUTHORIZATION};
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};
use std::{convert::Infallible, sync::Arc};
use subtle::ConstantTimeEq;
use tokio::{net::TcpListener, task::JoinSet};
use tokio_util::sync::CancellationToken;

type Sdk = StreamableHttpService<Endpoint, NeverSessionManager>;
type HttpResponse = Response<BoxBody<Bytes, Infallible>>;

pub(super) async fn run(
    listener: TcpListener,
    endpoint: Endpoint,
    authorization: Arc<str>,
    cancellation: CancellationToken,
    options: HttpOptions,
    body_bytes: usize,
) -> Result<(), HttpError> {
    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts([options.address.to_string()])
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_max_request_body_bytes(body_bytes)
        .with_cancellation_token(cancellation.clone())
        .enforce_origin_validation();
    let handler = endpoint.clone();
    let sdk = Sdk::new(move || Ok(handler.clone()), Arc::default(), config);
    let mut connections = JoinSet::new();
    let mut result = loop {
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => break Ok(()),
            _ = endpoint.cancelled() => break Ok(()),
            Some(outcome) = connections.join_next(), if !connections.is_empty() => { if outcome.is_err() { break Err(HttpError::Executor); } },
            accepted = listener.accept() => {
                let (socket, _) = match accepted {
                    Ok(accepted) => accepted,
                    Err(error) => break Err(HttpError::from(error)),
                };
                if connections.len() == options.connections {
                    drop(socket);
                    continue;
                }
                let sdk = sdk.clone();
                let authorization = authorization.clone();
                let timeout = options.request_timeout;
                connections.spawn(async move {
                    let service = service_fn(move |request| {
                        request_response(sdk.clone(), authorization.clone(), request)
                    });
                    let mut builder = http1::Builder::new();
                    builder.keep_alive(false).max_headers(64).max_buf_size(16 * 1024);
                    let connection = builder.serve_connection(TokioIo::new(socket), service);
                    let _ = tokio::time::timeout(timeout, connection).await;
                });
            },
        }
    };
    drop(listener);
    endpoint.close();
    cancellation.cancel();
    if tokio::time::timeout(options.shutdown_timeout, async {
        while let Some(outcome) = connections.join_next().await {
            if outcome.is_err() {
                result = Err(HttpError::Executor);
            }
        }
    })
    .await
    .is_err()
    {
        connections.abort_all();
        while connections.join_next().await.is_some() {}
        return Err(HttpError::ShutdownTimeout);
    }
    result
}

async fn request_response(
    sdk: Sdk,
    authorization: Arc<str>,
    request: Request<Incoming>,
) -> Result<HttpResponse, Infallible> {
    if request.uri().path() != "/mcp" || request.uri().query().is_some() {
        return Ok(status(StatusCode::NOT_FOUND));
    }
    let headers = request.headers().get_all(AUTHORIZATION);
    let mut values = headers.iter();
    let authorized = values
        .next()
        .is_some_and(|value| bool::from(value.as_bytes().ct_eq(authorization.as_bytes())))
        && values.next().is_none();
    if !authorized {
        let mut response = status(StatusCode::UNAUTHORIZED);
        response.headers_mut().insert(
            http::header::WWW_AUTHENTICATE,
            http::HeaderValue::from_static("Bearer"),
        );
        return Ok(response);
    }
    Ok(sdk.handle(request).await)
}

fn status(code: StatusCode) -> HttpResponse {
    let mut response = Response::new(Full::new(Bytes::new()).boxed());
    *response.status_mut() = code;
    response
}

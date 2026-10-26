#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Native lifecycle fixtures"
)]
//! Real executor ownership covers abandoned openings and embedding under Tokio.

use fpas_mcp::{Catalog, Fault, HttpOptions, HttpRuntime, Limits, Server};
use std::time::Duration;

fn server() -> Server {
    Server::new(
        Catalog::new(Limits {
            endpoints: 1,
            ..Limits::default()
        })
        .unwrap(),
    )
}

#[tokio::test]
async fn abandoned_opening_releases_endpoint_capacity_and_executor_drop_preserves_borrowed_server()
{
    let server = server();
    let runtime = HttpRuntime::new().unwrap();
    let ticket = runtime
        .publish(server.clone(), HttpOptions::default())
        .unwrap();
    let cleanup = ticket.cleanup_receipt();
    drop(ticket);
    tokio::time::timeout(Duration::from_secs(5), cleanup.wait())
        .await
        .unwrap()
        .unwrap();
    let mut replacement = runtime
        .publish_attached(server.clone(), HttpOptions::default(), 41)
        .unwrap();
    let publication = tokio::time::timeout(Duration::from_secs(5), replacement.complete())
        .await
        .unwrap()
        .unwrap();
    publication.bind_session("trusted-session").unwrap();
    assert_eq!(
        publication.bind_session("redirected"),
        Err(Fault::InvalidArgument("Attachment"))
    );
    drop(runtime);
    assert!(publication.cleanup_result().unwrap().is_ok());
    let independent = server.endpoint().unwrap();
    independent.close();
    server.close();
}

#[tokio::test]
async fn server_close_joins_opening_receipts_and_prevents_late_publication() {
    let server = server();
    let runtime = HttpRuntime::new().unwrap();
    let mut ticket = runtime
        .publish(server.clone(), HttpOptions::default())
        .unwrap();
    let original = ticket.cleanup_receipt();
    let (first, receipts) = runtime.stop_server(&server);
    assert!(first);
    for receipt in receipts {
        tokio::time::timeout(Duration::from_secs(5), receipt.wait())
            .await
            .unwrap()
            .unwrap();
    }
    tokio::time::timeout(Duration::from_secs(5), original.wait())
        .await
        .unwrap()
        .unwrap();
    if let Ok(publication) = ticket.complete().await {
        assert!(!publication.stop());
        assert!(publication.cleanup_result().unwrap().is_ok());
    }
    assert!(runtime.publish(server, HttpOptions::default()).is_err());
}

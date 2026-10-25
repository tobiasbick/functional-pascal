//! Owned generic transports must remain closeable while physical output is blocked.

use fpas_acp::{
    RawJson,
    connection::{Fault, Limits, Payload, Runtime},
    schema::Version,
    transport::{Frame, Transport, TransportStatus},
};
use futures::future::BoxFuture;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct Blocked {
    entered: AtomicBool,
    finished: AtomicBool,
    resume: tokio::sync::Notify,
}
impl Transport for Blocked {
    fn receive(&self) -> BoxFuture<'_, Result<Option<Frame>, Fault>> {
        Box::pin(std::future::pending())
    }
    fn send(&self, _frame: RawJson) -> BoxFuture<'_, Result<(), Fault>> {
        Box::pin(async {
            self.entered.store(true, Ordering::Release);
            self.resume.notified().await;
            Ok(())
        })
    }
    fn finish(&self, _grace_millis: u64) -> BoxFuture<'_, Result<TransportStatus, Fault>> {
        Box::pin(async {
            self.finished.store(true, Ordering::Release);
            Ok(TransportStatus::default())
        })
    }
}

#[test]
fn close_interrupts_a_blocked_owned_transport_without_waiting_for_the_peer() {
    let runtime = Runtime::new().unwrap();
    let transport = Arc::new(Blocked::default());
    let connection = runtime
        .connect(Version::V1, Limits::default(), transport.clone())
        .unwrap();
    let token = connection
        .send("_blocked", RawJson::parse("{}").unwrap())
        .unwrap();
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    executor.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while !transport.entered.load(Ordering::Acquire) { tokio::task::yield_now().await; }
        }).await.unwrap();
        connection.close().unwrap();
        let reader = connection.reader().unwrap();
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(2), reader.receive()).await;
        // Unblock even on failure so fixture teardown cannot hide the assertion.
        transport.resume.notify_one();
        let event = outcome.expect("local close must interrupt physical output").unwrap();
        assert!(matches!(event.payload, Payload::RequestFailed(actual, Fault::Closed) if actual == token));
        assert!(matches!(reader.receive().await.unwrap().payload, Payload::Closed(_)));
    });
    assert!(transport.finished.load(Ordering::Acquire));
}

struct CleanupFailure;
impl Transport for CleanupFailure {
    fn receive(&self) -> BoxFuture<'_, Result<Option<Frame>, Fault>> {
        Box::pin(async { Ok(None) })
    }
    fn send(&self, _frame: RawJson) -> BoxFuture<'_, Result<(), Fault>> {
        Box::pin(async { Ok(()) })
    }
    fn finish(&self, _grace_millis: u64) -> BoxFuture<'_, Result<TransportStatus, Fault>> {
        Box::pin(async { Err(Fault::Host) })
    }
}

#[test]
fn cleanup_failure_retains_ownership_and_prevents_successful_release() {
    let runtime = Runtime::new().unwrap();
    let connection = runtime
        .connect(Version::V1, Limits::default(), Arc::new(CleanupFailure))
        .unwrap();
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let reader = connection.reader().unwrap();
    let event = executor.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(10), reader.receive())
            .await
            .unwrap()
            .unwrap()
    });
    assert!(
        matches!(event.payload, Payload::Closed(status) if status.cleanup_fault == Some(Fault::Host))
    );
    assert_eq!(connection.cleanup_result(), Err(Fault::Host));
    assert_eq!(connection.release(), Err(Fault::Host));
    assert_eq!(
        connection.state().unwrap(),
        fpas_acp::connection::Lifecycle::Failed
    );
}

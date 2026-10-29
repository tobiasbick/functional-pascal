#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
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

#[test]
fn startup_reservations_bound_concurrent_constructors_and_shutdown_does_not_wait_for_them() {
    use fpas_acp::connection::Settings;
    use std::{sync::mpsc, time::Duration};
    let runtime = Arc::new(Runtime::new().unwrap());
    let (entered, entries) = mpsc::channel();
    let mut releases = Vec::new();
    let mut threads = Vec::new();
    let transports = (0..16)
        .map(|_| Arc::new(Blocked::default()))
        .collect::<Vec<_>>();
    for transport in &transports {
        let runtime = runtime.clone();
        let entered = entered.clone();
        let transport = transport.clone();
        let (release, released) = mpsc::channel();
        releases.push(release);
        threads.push(std::thread::spawn(move || {
            runtime.connect_configured_with(
                Settings::for_version(Version::V1),
                Limits::default(),
                || {
                    entered.send(()).unwrap();
                    released.recv_timeout(Duration::from_secs(10)).unwrap();
                    Ok(transport)
                },
            )
        }));
    }
    let all_entered = (0..16).all(|_| entries.recv_timeout(Duration::from_secs(3)).is_ok());
    let rejected = if all_entered {
        Some(runtime.connect_configured_with(
            Settings::for_version(Version::V1),
            Limits::default(),
            || panic!("full admission must precede construction"),
        ))
    } else {
        None
    };
    let (stopped, receipt) = mpsc::channel();
    let stopping = runtime.clone();
    let stopper = std::thread::spawn(move || {
        stopping.request_shutdown();
        stopped.send(()).unwrap();
    });
    let signalled = receipt.recv_timeout(Duration::from_secs(3));
    for release in releases {
        let _ = release.send(());
    }
    let results = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    stopper.join().unwrap();
    runtime.shutdown();
    assert!(
        all_entered,
        "constructors serialized behind the admission lock"
    );
    assert!(matches!(
        rejected,
        Some(Err(Fault::LimitExceeded("Connections", 16)))
    ));
    signalled.expect("shutdown must return while constructors are still gated");
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(Fault::Closed)))
    );
    assert!(
        transports
            .iter()
            .all(|transport| transport.finished.load(Ordering::Acquire))
    );
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

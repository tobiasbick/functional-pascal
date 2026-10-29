//! Abandonment retires native work and accepted sockets that never reached FPAS.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "native socket lifecycle fixtures"
)]

use super::*;
use std::{
    io::Read,
    net::TcpStream,
    time::{Duration, Instant},
};

#[test]
fn dropping_a_completed_outgoing_connect_closes_the_undelivered_connection() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let request = ConnectRequest::new(
        "127.0.0.1",
        i64::from(listener.local_addr().unwrap().port()),
        5000,
        ConnectMode::Tcp,
    )
    .unwrap();
    let mut job = Job::start(hosted.clone(), Operation::Connect(request), None).unwrap();
    let (mut peer, _) = listener.accept().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    job.thread.take().unwrap().join().unwrap();
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
    drop(job);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert_eq!(peer.read(&mut [0]).unwrap(), 0);
}

#[test]
#[allow(
    clippy::expect_used,
    reason = "test setup and native peer operations must succeed"
)]
fn dropping_wait_joins_pending_accept_and_closes_an_undelivered_connection() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = hosted
        .network_listeners
        .listen("127.0.0.1", 0)
        .expect("listener");
    let address = hosted
        .network_listeners
        .local_address(listener)
        .expect("address");
    let pending = Job::start(Arc::clone(&hosted), Operation::Accept(listener), None).expect("job");
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
    let started = Instant::now();
    drop(pending);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert!(started.elapsed() < Duration::from_secs(1));
    let mut accepted =
        Job::start(Arc::clone(&hosted), Operation::Accept(listener), None).expect("job");
    let mut peer = TcpStream::connect(address).expect("peer");
    peer.set_read_timeout(Some(Duration::from_secs(1)))
        .expect("timeout");
    accepted
        .thread
        .take()
        .expect("worker")
        .join()
        .expect("finished accept");
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        1,
        "completed undelivered results still consume their reservation"
    );
    drop(accepted);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert_eq!(peer.read(&mut [0]).expect("abandoned connection closes"), 0);
    hosted
        .network_listeners
        .close(listener)
        .expect("close listener");
}

#[test]
fn native_socket_admission_is_bounded_per_vm_and_recovers_after_release() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = hosted
        .network_listeners
        .listen("127.0.0.1", 0)
        .expect("listener");
    let mut permits = (0..MAX_PENDING_JOBS)
        .map(|_| Permit::acquire(hosted.clone()).expect("available reservation"))
        .collect::<Vec<_>>();
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        MAX_PENDING_JOBS
    );
    match Job::start(hosted.clone(), Operation::Accept(listener), None) {
        Err(Failure::Native {
            kind: super::super::failure::NativeKind::IoFailure,
            ..
        }) => {}
        _ => panic!("saturation must fail before starting a native thread"),
    }
    let other = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    drop(Permit::acquire(other).expect("another VM has an independent budget"));
    drop(permits.pop());
    let admitted =
        Job::start(hosted.clone(), Operation::Accept(listener), None).expect("reused slot");
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        MAX_PENDING_JOBS
    );
    drop(admitted);
    drop(permits);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    hosted.network_listeners.close(listener).expect("close");
}

#[test]
fn abandoning_a_pending_read_joins_without_closing_its_connection() {
    use std::io::Write;
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let connection = hosted
        .network_connections
        .connect_tcp(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            5000,
        )
        .expect("connection");
    let (mut peer, _) = listener.accept().expect("peer");
    hosted
        .network_connections
        .set_timeout(connection, 5000)
        .expect("timeout");
    let job = Job::start(hosted.clone(), Operation::Read(connection, 1), None).expect("read");
    drop(job);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    peer.write_all(b"x").expect("send after abandonment");
    assert_eq!(
        hosted
            .network_connections
            .read(connection, 1)
            .expect("connection remains usable"),
        b"x"
    );
    hosted.network_connections.close(connection).expect("close");
}

//! Abandonment retires native work and accepted sockets that never reached FPAS.

use super::*;
use std::{
    io::Read,
    net::TcpStream,
    time::{Duration, Instant},
};

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
    let started = Instant::now();
    drop(pending);
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
    drop(accepted);
    assert_eq!(peer.read(&mut [0]).expect("abandoned connection closes"), 0);
    hosted
        .network_listeners
        .close(listener)
        .expect("close listener");
}

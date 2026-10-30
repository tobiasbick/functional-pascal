//! Real TCP controls keep the child-termination observer's negative cases meaningful.

use super::observer::{Observer, assert_stream_closed};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

fn pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let peer = TcpStream::connect_timeout(&listener.local_addr().unwrap(), Duration::from_secs(1))
        .unwrap();
    (listener.accept().unwrap().0, peer)
}

#[test]
fn orderly_peer_eof_is_closed() {
    let (stream, peer) = pair();
    drop(peer);
    assert_stream_closed(stream, Instant::now() + Duration::from_secs(1));
}

#[test]
fn abortive_peer_close_is_closed() {
    let (stream, peer) = pair();
    let peer = socket2::Socket::from(peer);
    peer.set_linger(Some(Duration::ZERO)).unwrap();
    drop(peer);
    assert_stream_closed(stream, Instant::now() + Duration::from_secs(1));
}

#[test]
fn live_peer_is_rejected_after_the_absolute_read_deadline() {
    let (stream, _peer) = pair();
    stream.set_nonblocking(true).unwrap();
    let started = Instant::now();
    let outcome = std::panic::catch_unwind(|| {
        assert_stream_closed(stream, started + Duration::from_millis(40));
    });
    assert!(outcome.is_err(), "a live peer must not count as terminated");
    assert!(
        started.elapsed() >= Duration::from_millis(20),
        "accepted sockets must restore blocking reads"
    );
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "observer read exceeded its bounded deadline"
    );
}

#[test]
fn peer_data_is_rejected_as_evidence_of_termination() {
    let (stream, mut peer) = pair();
    peer.write_all(b"x").unwrap();
    assert!(
        std::panic::catch_unwind(|| {
            assert_stream_closed(stream, Instant::now() + Duration::from_secs(1));
        })
        .is_err(),
        "a peer sending data is still alive"
    );
}

#[test]
fn missing_connection_is_rejected_after_the_accept_deadline() {
    let observer = Observer::with_timeout(Duration::from_millis(40));
    let started = Instant::now();
    assert!(std::panic::catch_unwind(|| observer.assert_closed(1)).is_err());
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "observer accept exceeded its bounded deadline"
    );
}

use std::cell::Cell;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use super::{connect, in_progress, wait_for_connection};

#[test]
fn pending_attempt_observes_cancellation_without_waiting_for_deadline() {
    let checks = Cell::new(0);
    let started = Instant::now();
    let result = wait_for_connection(
        started + Duration::from_secs(5),
        || checks.get() >= 2,
        || {
            checks.set(checks.get() + 1);
            Ok(false)
        },
    );
    assert_eq!(result, Err("Network connect cancelled".into()));
    assert_eq!(checks.get(), 2);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn pending_attempt_expires_without_restarting_its_budget() {
    let started = Instant::now();
    let result = wait_for_connection(
        started + Duration::from_millis(30),
        || started.elapsed() > Duration::from_secs(2),
        || Ok(false),
    );
    assert_eq!(result, Err("Network connect timed out".into()));
}

#[test]
fn cancellation_wins_over_simultaneous_readiness() {
    let cancelled = Cell::new(false);
    assert_eq!(
        wait_for_connection(
            Instant::now() + Duration::from_secs(1),
            || cancelled.get(),
            || {
                cancelled.set(true);
                Ok(true)
            }
        ),
        Err("Network connect cancelled".into())
    );
}

#[test]
fn socket_errors_are_not_retried_as_pending() {
    let mut checks = 0;
    let result = wait_for_connection(
        Instant::now() + Duration::from_secs(1),
        || false,
        || {
            checks += 1;
            Err(io::ErrorKind::ConnectionRefused.into())
        },
    );
    assert!(result.expect_err("refused").contains("TCP connect failed"));
    assert_eq!(checks, 1);
    assert!(!in_progress(&io::ErrorKind::ConnectionRefused.into()));
}

#[test]
fn connected_socket_is_restored_to_blocking_io() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let mut stream = connect(
        listener.local_addr().expect("address"),
        Instant::now() + Duration::from_secs(2),
        || false,
    )
    .expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("timeout");
    let (mut peer, _) = listener.accept().expect("accept");
    let writer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        peer.write_all(b"ok").expect("write");
    });
    let mut bytes = [0; 2];
    let result = stream.read_exact(&mut bytes);
    writer.join().expect("writer");
    result.expect("blocking read waited for data");
    assert_eq!(&bytes, b"ok");
}

#[cfg(unix)]
#[test]
fn unix_in_progress_is_pending() {
    assert!(in_progress(&io::Error::from_raw_os_error(
        libc::EINPROGRESS
    )));
}

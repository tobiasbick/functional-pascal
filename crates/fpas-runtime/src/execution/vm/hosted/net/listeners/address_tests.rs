//! Dynamic listener address and handle-lifetime regressions.

use super::NetworkListeners;
use crate::execution::vm::hosted::net::failure::Failure;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

#[test]
fn assigned_address_stays_reserved_and_accepts_connections() {
    let listeners = NetworkListeners::new();
    let first = listeners.listen("127.0.0.1", 0).expect("first listener");
    let second = listeners.listen("127.0.0.1", 0).expect("second listener");
    let address = listeners.local_address(first).expect("assigned address");
    assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_ne!(address.port(), 0);
    assert_ne!(
        address,
        listeners.local_address(second).expect("second address")
    );
    let _client = TcpStream::connect(address).expect("connect to assigned address");
    let _accepted = listeners.accept(first).expect("accept connection");
    assert_eq!(
        listeners.local_address(first).expect("stable address"),
        address
    );
    listeners.close(first).expect("close first");
    assert!(listeners.local_address(first).is_err());
    listeners.shutdown();
    assert!(listeners.local_address(second).is_err());
}

#[test]
fn listener_address_rejects_invalid_handles_and_ports() {
    let listeners = NetworkListeners::new();
    assert!(listeners.local_address(0).is_err());
    for port in [-1, 65536, i64::MAX] {
        assert!(matches!(
            listeners
                .listen("127.0.0.1", port)
                .expect_err("invalid port"),
            Failure::InvalidArgument { name: "Port", .. }
        ));
    }
}

#[test]
fn wildcard_listener_reports_the_bound_address() {
    let listeners = NetworkListeners::new();
    let handle = listeners.listen("0.0.0.0", 0).expect("wildcard listener");
    let address = listeners.local_address(handle).expect("address");
    assert!(address.ip().is_unspecified());
    assert_ne!(address.port(), 0);
}

#[test]
fn cancelled_listener_configuration_does_not_resolve_or_load_credentials() {
    let listeners = NetworkListeners::new();
    assert_eq!(
        listeners.listen_with_cancellation("unused.invalid", 0, || true),
        Err(Failure::Cancelled)
    );
    assert_eq!(
        listeners.listen_tls_with_cancellation(
            "unused.invalid",
            0,
            "missing-cert.pem",
            "missing-key.pem",
            5000,
            || true
        ),
        Err(Failure::Cancelled)
    );
    assert!(listeners.listeners.lock().unwrap().is_empty());
}

#[test]
fn cancellation_before_listener_publication_releases_the_socket() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    use std::time::{Duration, Instant};
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = probe.local_addr().unwrap();
    drop(probe);
    let listeners = Arc::new(NetworkListeners::new());
    let registry = listeners.listeners.lock().unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let stop = cancelled.clone();
    let state = listeners.clone();
    let (send, receive) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let result = state.listen_with_cancellation("127.0.0.1", i64::from(address.port()), || {
            stop.load(Ordering::Acquire)
        });
        let _ = send.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    let bound = loop {
        if let Ok(peer) = TcpStream::connect(address) {
            drop(peer);
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        std::thread::park_timeout(Duration::from_millis(1));
    };
    cancelled.store(true, Ordering::Release);
    drop(registry);
    thread.join().unwrap();
    assert!(bound, "native bind did not reach publication");
    assert_eq!(
        receive.recv_timeout(Duration::from_secs(5)).unwrap(),
        Err(Failure::Cancelled)
    );
    assert!(listeners.listeners.lock().unwrap().is_empty());
    let _rebound = std::net::TcpListener::bind(address).unwrap();
}

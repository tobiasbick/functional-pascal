//! Dynamic listener address and handle-lifetime regressions.

use super::NetworkListeners;
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
        assert!(
            listeners
                .listen("127.0.0.1", port)
                .expect_err("invalid port")
                .contains("0..=65535")
        );
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

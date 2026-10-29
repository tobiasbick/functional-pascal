//! Controlled OS calls prove abandonment without relying on external DNS timing.

use super::*;
use std::sync::Arc;

#[test]
fn numeric_addresses_use_the_supplied_port_without_dns() {
    for host in ["127.0.0.1", "::1"] {
        let result = resolve(host, 43210, Instant::now() + Duration::from_secs(5), || {
            false
        })
        .expect("numeric address");
        assert_eq!(result, vec![SocketAddr::new(host.parse().unwrap(), 43210)]);
    }
}

#[test]
fn local_name_resolution_joins_and_preserves_the_port() {
    let result = resolve(
        "localhost",
        43210,
        Instant::now() + Duration::from_secs(5),
        || false,
    )
    .expect("local hostname");
    assert!(!result.is_empty());
    assert!(
        result
            .iter()
            .all(|address| address.ip().is_loopback() && address.port() == 43210)
    );
}

#[test]
fn resolver_budget_stays_reserved_until_the_last_permit_drops() {
    let active = AtomicUsize::new(0);
    let mut permits = (0..MAX_LOOKUPS)
        .map(|_| Permit::acquire(&active).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        Permit::acquire(&active),
        Err(Failure::Native {
            kind: crate::execution::vm::hosted::net::failure::NativeKind::IoFailure,
            code: None,
            ..
        })
    ));
    drop(permits.pop());
    let replacement = Permit::acquire(&active).unwrap();
    assert_eq!(active.load(Ordering::Acquire), MAX_LOOKUPS);
    drop(permits);
    drop(replacement);
    assert_eq!(active.load(Ordering::Acquire), 0);
}

#[test]
fn cancelled_and_expired_lookups_discard_late_addresses_without_opening_a_socket() {
    for cancel in [false, true] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let input = Arc::new("owned native hostname".to_owned());
        let weak = Arc::downgrade(&input);
        let (entered, receipt) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let mut lookup = Lookup::start(move || {
            entered.send(()).unwrap();
            released.recv().unwrap();
            assert!(!input.is_empty());
            Ok(vec![address])
        })
        .unwrap();
        let worker = lookup.thread.take().unwrap();
        receipt.recv_timeout(Duration::from_secs(5)).unwrap();
        let checks = AtomicUsize::new(0);
        let duration = if cancel {
            Duration::from_secs(5)
        } else {
            Duration::from_millis(1)
        };
        let outcome = lookup.wait(Some(Instant::now() + duration), || {
            cancel && checks.fetch_add(1, Ordering::AcqRel) != 0
        });
        assert_eq!(
            outcome,
            Err(if cancel {
                Failure::Cancelled
            } else {
                Failure::deadline("Network connect timed out")
            })
        );
        assert!(
            !worker.is_finished(),
            "abandonment must not wait for OS resolution"
        );
        assert!(
            weak.upgrade().is_some(),
            "only the native input remains alive"
        );
        release.send(()).unwrap();
        worker.join().unwrap();
        assert!(weak.upgrade().is_none());
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }
}

#[test]
fn cancellation_after_dns_completion_wins_over_addresses_and_errors() {
    for outcome in [
        Ok(vec!["127.0.0.1:1".parse().unwrap()]),
        Err(io::Error::other("resolution failed")),
    ] {
        let mut lookup = Lookup::start(move || outcome).unwrap();
        lookup.thread.take().unwrap().join().unwrap();
        let checks = AtomicUsize::new(0);
        assert_eq!(
            lookup.wait(Some(Instant::now() + Duration::from_secs(5)), || checks
                .fetch_add(1, Ordering::AcqRel)
                != 0),
            Err(Failure::Cancelled)
        );
    }
}

#[test]
fn successful_lookup_releases_native_input_before_returning() {
    let input = Arc::new("hostname".to_owned());
    let weak = Arc::downgrade(&input);
    let lookup = Lookup::start(move || {
        assert!(!input.is_empty());
        Ok(vec!["127.0.0.1:1".parse().unwrap()])
    })
    .unwrap();
    assert_eq!(
        lookup
            .wait(Some(Instant::now() + Duration::from_secs(5)), || false)
            .unwrap()
            .len(),
        1
    );
    assert!(weak.upgrade().is_none());
}

#[test]
fn resolver_panic_becomes_a_native_failure() {
    let lookup = Lookup::start(|| panic!("OS resolver fixture failed")).unwrap();
    assert!(matches!(
        lookup.wait(Some(Instant::now() + Duration::from_secs(5)), || false),
        Err(Failure::Native {
            kind: crate::execution::vm::hosted::net::failure::NativeKind::IoFailure,
            ..
        })
    ));
}

#[test]
fn listener_lookups_preserve_ephemeral_ports_and_cancel_without_a_deadline() {
    for host in ["127.0.0.1", "::1", "localhost"] {
        let addresses = resolve_listener(host, 0, || false).unwrap();
        assert!(!addresses.is_empty());
        assert!(
            addresses
                .iter()
                .all(|address| address.ip().is_loopback() && address.port() == 0)
        );
    }
    let (release, released) = mpsc::channel();
    let mut lookup = Lookup::start(move || {
        released.recv().unwrap();
        Ok(vec!["127.0.0.1:0".parse().unwrap()])
    })
    .unwrap();
    let thread = lookup.thread.take().unwrap();
    let result = lookup.wait(None, || true);
    release.send(()).unwrap();
    thread.join().unwrap();
    assert_eq!(result, Err(Failure::Cancelled));
}

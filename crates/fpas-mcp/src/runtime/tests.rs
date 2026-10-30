//! Gated construction proves admission, shutdown and failure delivery before setup completes.
#![allow(
    clippy::unwrap_used,
    reason = "Native lifecycle fixtures use direct assertions"
)]

use super::*;
use std::{sync::mpsc, time::Duration};

fn server() -> Server {
    Server::new(crate::Catalog::new(Default::default()).unwrap())
}
fn builder() -> Result<tokio::runtime::Runtime, HttpError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Into::into)
}

#[test]
fn construction_queues_bounded_openings_and_shutdown_signals_before_setup_returns() {
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let runtime = HttpRuntime::with_builder(move || {
        entered.send(()).unwrap();
        released.recv_timeout(Duration::from_secs(5)).unwrap();
        builder()
    })
    .unwrap();
    entry.recv_timeout(Duration::from_secs(5)).unwrap();
    let server = server();
    let mut tickets = (0..OPEN_QUEUE)
        .map(|_| runtime.publish(server.clone(), Default::default()).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        runtime.publish(server.clone(), Default::default()),
        Err(HttpError::Server(Fault::Busy))
    ));
    assert!(tickets.iter_mut().all(|t| t.try_complete().is_none()));
    runtime.request_shutdown();
    assert!(matches!(
        runtime.publish(server, Default::default()),
        Err(HttpError::Server(Fault::Closed))
    ));
    release.send(()).unwrap();
    runtime.wait_closed();
    for ticket in &mut tickets {
        assert!(matches!(
            ticket.try_complete(),
            Some(Err(HttpError::Server(Fault::Closed)))
        ));
        assert_eq!(ticket.cleanup_receipt().result(), Some(Ok(())));
    }
    assert_eq!(runtime.state.lock().unwrap().opening, 0);
}

#[test]
fn setup_errors_and_panics_complete_every_queued_ticket_without_leaking_capacity() {
    for panic in [false, true] {
        let error = HttpError::Io(std::io::ErrorKind::PermissionDenied);
        let expected = if panic {
            HttpError::Executor
        } else {
            error.clone()
        };
        let (release, released) = mpsc::channel();
        let runtime = HttpRuntime::with_builder(move || {
            released.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(!panic, "controlled executor construction panic");
            Err(error)
        })
        .unwrap();
        let server = server();
        let mut tickets = (0..OPEN_QUEUE)
            .map(|_| runtime.publish(server.clone(), Default::default()).unwrap())
            .collect::<Vec<_>>();
        // Dropping an opening during setup must also finish its cleanup receipt.
        let abandoned = tickets.pop().unwrap();
        let cleanup = abandoned.cleanup_receipt();
        drop(abandoned);
        release.send(()).unwrap();
        runtime.wait_closed();
        assert_eq!(cleanup.result(), Some(Ok(())));
        for ticket in &mut tickets {
            assert!(matches!(ticket.try_complete(), Some(Err(e)) if e == expected));
            assert_eq!(ticket.cleanup_receipt().result(), Some(Ok(())));
        }
        assert!(matches!(runtime.publish(server, Default::default()), Err(e) if e == expected));
        assert_eq!(runtime.state.lock().unwrap().opening, 0);
    }
}

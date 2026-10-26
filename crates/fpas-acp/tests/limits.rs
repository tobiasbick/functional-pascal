#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Public resource contract boundaries, including retained closed-handle slots.

use fpas_acp::{
    connection::{Fault, Limits, Payload, Runtime},
    schema::Version,
};

#[test]
fn every_configurable_ceiling_rejects_one_past_each_boundary() {
    type Ceiling = (&'static str, fn(&mut Limits) -> &mut usize, usize, usize);
    let ceilings: &[Ceiling] = &[
        (
            "OutgoingRequests",
            |value| &mut value.outgoing_requests,
            1,
            4096,
        ),
        (
            "IncomingRequests",
            |value| &mut value.incoming_requests,
            1,
            4096,
        ),
        ("Sessions", |value| &mut value.sessions, 1, 4096),
        ("Terminals", |value| &mut value.terminals, 1, 256),
        ("Routes", |value| &mut value.routes, 1, 4096),
        ("QueueEntries", |value| &mut value.queue_entries, 1, 4096),
        (
            "OutboundEntries",
            |value| &mut value.outbound_entries,
            1,
            4096,
        ),
        (
            "QueueBytes",
            |value| &mut value.queue_bytes,
            1,
            64 * 1024 * 1024,
        ),
        (
            "MessageBytes",
            |value| &mut value.message_bytes,
            1024,
            64 * 1024 * 1024,
        ),
        (
            "StderrBytes",
            |value| &mut value.stderr_bytes,
            0,
            4 * 1024 * 1024,
        ),
        (
            "TerminalBytes",
            |value| &mut value.terminal_bytes,
            0,
            16 * 1024 * 1024,
        ),
        (
            "AllTerminalBytes",
            |value| &mut value.all_terminal_bytes,
            0,
            64 * 1024 * 1024,
        ),
    ];
    for &(name, select, minimum, maximum) in ceilings {
        for valid in [minimum, maximum] {
            let mut limits = Limits::default();
            *select(&mut limits) = valid;
            assert_eq!(limits.validate(), Ok(()), "{name}={valid}");
        }
        for invalid in [minimum.checked_sub(1), Some(maximum + 1)]
            .into_iter()
            .flatten()
        {
            let mut limits = Limits::default();
            *select(&mut limits) = invalid;
            assert_eq!(limits.validate(), Err(Fault::InvalidArgument(name)));
        }
    }
    for valid in [0, 60000] {
        assert_eq!(
            Limits {
                shutdown_millis: valid,
                ..Limits::default()
            }
            .validate(),
            Ok(())
        );
    }
    assert_eq!(
        Limits {
            shutdown_millis: 60001,
            ..Limits::default()
        }
        .validate(),
        Err(Fault::InvalidArgument("ShutdownMillis"))
    );
}

#[test]
fn closed_connections_keep_their_slot_until_release_then_return_it_once() {
    let runtime = Runtime::new().unwrap();
    let mut connections = Vec::new();
    for _ in 0..16 {
        connections.push(
            runtime
                .connect_messages(Version::V1, Limits::default())
                .unwrap(),
        );
    }
    assert!(matches!(
        runtime.connect_messages(Version::V1, Limits::default()),
        Err(Fault::LimitExceeded("Connections", 16))
    ));
    let (first, _) = &connections[0];
    first.close().unwrap();
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let reader = first.reader().unwrap();
    executor.block_on(async {
        let event = tokio::time::timeout(std::time::Duration::from_secs(10), reader.receive())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(event.payload, Payload::Closed(_)));
    });
    assert!(matches!(
        runtime.connect_messages(Version::V1, Limits::default()),
        Err(Fault::LimitExceeded("Connections", 16))
    ));
    assert!(first.release().unwrap());
    assert_eq!(first.release(), Err(Fault::Closed));
    let _replacement = runtime
        .connect_messages(Version::V1, Limits::default())
        .unwrap();
    assert!(matches!(
        runtime.connect_messages(Version::V1, Limits::default()),
        Err(Fault::LimitExceeded("Connections", 16))
    ));
}

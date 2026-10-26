#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Frame boundaries, bytes and alias ownership at the production message interface.

use fpas_acp::{
    RawJson,
    connection::{Fault, Limits},
    transport::{Framer, Transport, messages},
};

#[test]
fn fragmented_utf8_and_multiple_lines_preserve_complete_original_messages() {
    let input = "{\"text\":\"Grüße\",\"n\":1e400}\n{\"next\":true}\n".as_bytes();
    for split in 0..=input.len() {
        let mut framer = Framer::new(1024);
        let mut frames = framer.push(&input[..split]).unwrap();
        frames.extend(framer.push(&input[split..]).unwrap());
        assert_eq!(
            frames.iter().map(RawJson::text).collect::<Vec<_>>(),
            ["{\"text\":\"Grüße\",\"n\":1e400}", "{\"next\":true}"]
        );
        assert_eq!(framer.finish(), Ok(()));
    }
    let mut framer = Framer::new(2);
    assert_eq!(framer.push(b"{}\n").unwrap()[0].text(), "{}");
    assert!(matches!(
        framer.push(b"123"),
        Err(Fault::LimitExceeded("MessageBytes", 2))
    ));
    let mut framer = Framer::new(20);
    framer.push(b"{}").unwrap();
    assert_eq!(framer.finish(), Err(Fault::InvalidMessage));
    assert_eq!(Framer::new(20).push(b"\xff\n"), Err(Fault::InvalidMessage));
    let mut framer = Framer::new(20);
    assert_eq!(framer.push(b"{}\ninvalid\n").unwrap()[0].text(), "{}");
    assert_eq!(framer.failure(), Some(&Fault::InvalidMessage));
    assert_eq!(framer.push(b"{}\n"), Err(Fault::InvalidMessage));
}

#[test]
fn message_capacity_is_held_until_dispatch_credit_is_dropped_and_eof_is_atomic() {
    let limits = Limits {
        queue_entries: 1,
        queue_bytes: 2,
        ..Limits::default()
    };
    let (endpoint, transport) = messages(&limits).unwrap();
    let frame = RawJson::parse("{}").unwrap();
    assert!(endpoint.feed(frame.clone()).unwrap());
    assert!(!endpoint.feed(frame.clone()).unwrap());
    assert!(!endpoint.feed(RawJson::parse("123").unwrap()).unwrap());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let retained = runtime.block_on(transport.receive()).unwrap().unwrap();
    assert!(!endpoint.feed(frame.clone()).unwrap());
    drop(retained);
    assert!(endpoint.feed(frame.clone()).unwrap());
    let alias = endpoint.clone();
    assert!(endpoint.close_input());
    assert!(!alias.close_input());
    assert_eq!(alias.feed(frame), Err(Fault::Closed));
    assert!(runtime.block_on(transport.receive()).unwrap().is_some());
    assert!(runtime.block_on(transport.receive()).unwrap().is_none());
    runtime.block_on(transport.finish(0)).unwrap();
    assert!(endpoint.output_closed());
}

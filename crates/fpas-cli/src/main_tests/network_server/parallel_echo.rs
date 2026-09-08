//! Exercise the checked-in parallel TCP tutorial with real loopback clients.

use std::io::{Read, Write};
use std::net::Shutdown;

use super::{connect_when_ready, finish_server, start_server, unused_port};

#[test]
fn parallel_echo_example_serves_second_client_and_cancels_idle_reads() {
    let port = unused_port();
    let source = include_str!("../../../../../examples/network/tcp_parallel_echo_server.fpas")
        .replace("18082", &port.to_string())
        .replace("10000", "2500")
        // This socket regression embeds the VM in the test runner. Process policy is
        // covered by disposable-process tests in fpas-vm, never by this thread.
        .replace("CreateLifetime(1000, true)", "CreateLifetime(1000, false)")
        .replace("Unwrap(ObserveSignals(Life));", "");
    let (cwd, server) = start_server("parallel-echo-example", source);
    let mut idle = connect_when_ready(port);
    idle.write_all(b"ready").expect("prime first connection");
    let mut ready = [0; 5];
    idle.read_exact(&mut ready)
        .expect("first worker must be serving");
    assert_eq!(&ready, b"ready");
    // Keep the first connection open: a serial server cannot answer this request.
    let mut active = connect_when_ready(port);
    let payload: Vec<u8> = (0..12000).map(|index| (index % 251) as u8).collect();
    active
        .write_all(&payload)
        .expect("write fragmented echo payload");
    active
        .shutdown(Shutdown::Write)
        .expect("finish client writes");
    let mut echoed = Vec::new();
    active.read_to_end(&mut echoed).expect("read complete echo");
    let mut stopped = Vec::new();
    idle.read_to_end(&mut stopped)
        .expect("shutdown releases idle client");
    let (stdout, stderr) = finish_server(cwd, server);
    assert_eq!(echoed, payload);
    assert!(stopped.is_empty());
    assert_eq!(
        stdout,
        "Parallel TCP echo ready\nParallel TCP echo stopped\n"
    );
    assert!(stderr.is_empty(), "{stderr}");
}

//! Exercise the checked-in parallel TCP tutorial with real loopback clients.

use std::io::{Read, Write};
use std::net::Shutdown;

use super::{connect_when_ready, finish_server, start_server};

#[test]
fn parallel_echo_example_serves_second_client_and_cancels_idle_reads() {
    let source = include_str!("../../../../../examples/network/tcp_parallel_echo_server.fpas")
        .replace("18082", "0")
        .replace("10000", "2500")
        .replace(
            "const ListenerValue: Net.Listener := Net.Listen('127.0.0.1', Port).Unwrap();",
            "const ListenerValue: Net.Listener := Net.Listen('127.0.0.1', Port).Unwrap();\n  Console.WriteLn('FPAS_SERVER_FIXTURE_PORT=', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port);",
        );
    let (cwd, server) = start_server("parallel-echo-example", source);
    let port = server.port();
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

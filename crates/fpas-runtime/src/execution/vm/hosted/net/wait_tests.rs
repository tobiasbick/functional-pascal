//! Abandonment retires native work and accepted sockets that never reached FPAS.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "native socket lifecycle fixtures"
)]

use super::super::connections::{ConnectMode, ConnectRequest};
use super::*;
use std::{
    io::Read,
    net::TcpStream,
    time::{Duration, Instant},
};

#[test]
fn native_callback_can_return_before_network_join_and_late_listener_is_closed() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let handle = hosted.network_listeners.listen("127.0.0.1", 0).unwrap();
    let (release, wait) = mpsc::channel();
    let (sender, receiver) = mpsc::channel();
    let job = Job {
        permit: Some(Permit::acquire(hosted.clone()).unwrap()),
        stop: Arc::new(AtomicBool::new(false)),
        receiver: Mutex::new(receiver),
        thread: Some(std::thread::spawn(move || {
            wait.recv_timeout(Duration::from_secs(10))
                .expect("host callback must return before joining network I/O");
            sender.send(Ok(Output::Listener(handle))).unwrap();
        })),
    };
    super::super::super::native::defer_joins(|| drop(job));
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
    release.send(()).unwrap();
    super::super::super::native::join_retired(&hosted);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert!(hosted.network_listeners.close(handle).is_err());
}

#[test]
fn dropping_a_completed_outgoing_connect_closes_the_undelivered_connection() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let request = ConnectRequest::new(
        "127.0.0.1",
        i64::from(listener.local_addr().unwrap().port()),
        5000,
        ConnectMode::Tcp,
    )
    .unwrap();
    let mut job = Job::start(hosted.clone(), Operation::Connect(request), None).unwrap();
    let (mut peer, _) = listener.accept().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    job.thread.take().unwrap().join().unwrap();
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
    drop(job);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert_eq!(peer.read(&mut [0]).unwrap(), 0);
}

#[test]
#[allow(
    clippy::expect_used,
    reason = "test setup and native peer operations must succeed"
)]
fn dropping_wait_joins_pending_accept_and_closes_an_undelivered_connection() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = hosted
        .network_listeners
        .listen("127.0.0.1", 0)
        .expect("listener");
    let address = hosted
        .network_listeners
        .local_address(listener)
        .expect("address");
    let pending = Job::start(Arc::clone(&hosted), Operation::Accept(listener), None).expect("job");
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
    let started = Instant::now();
    drop(pending);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert!(started.elapsed() < Duration::from_secs(1));
    let mut accepted =
        Job::start(Arc::clone(&hosted), Operation::Accept(listener), None).expect("job");
    let mut peer = TcpStream::connect(address).expect("peer");
    peer.set_read_timeout(Some(Duration::from_secs(1)))
        .expect("timeout");
    accepted
        .thread
        .take()
        .expect("worker")
        .join()
        .expect("finished accept");
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        1,
        "completed undelivered results still consume their reservation"
    );
    drop(accepted);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    assert_eq!(peer.read(&mut [0]).expect("abandoned connection closes"), 0);
    hosted
        .network_listeners
        .close(listener)
        .expect("close listener");
}

#[test]
fn native_socket_admission_is_bounded_per_vm_and_recovers_after_release() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = hosted
        .network_listeners
        .listen("127.0.0.1", 0)
        .expect("listener");
    let mut permits = (0..MAX_PENDING_JOBS)
        .map(|_| Permit::acquire(hosted.clone()).expect("available reservation"))
        .collect::<Vec<_>>();
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        MAX_PENDING_JOBS
    );
    match Job::start(hosted.clone(), Operation::Accept(listener), None) {
        Err(Failure::Native {
            kind: super::super::failure::NativeKind::IoFailure,
            ..
        }) => {}
        _ => panic!("saturation must fail before starting a native thread"),
    }
    let other = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    drop(Permit::acquire(other).expect("another VM has an independent budget"));
    drop(permits.pop());
    let admitted =
        Job::start(hosted.clone(), Operation::Accept(listener), None).expect("reused slot");
    assert_eq!(
        hosted.network_jobs.load(Ordering::Acquire),
        MAX_PENDING_JOBS
    );
    drop(admitted);
    drop(permits);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    hosted.network_listeners.close(listener).expect("close");
}

#[test]
fn abandoning_an_unpublished_listener_releases_its_bound_port() {
    use super::super::config_tests::Fixture;
    let fixture = Fixture::new();
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert = fixture.path("certificate.pem");
    let key = fixture.path("key.pem");
    std::fs::write(&cert, certificate.cert.pem()).unwrap();
    std::fs::write(&key, certificate.signing_key.serialize_pem()).unwrap();
    for tls in [false, true] {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = probe.local_addr().unwrap();
        drop(probe);
        let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
        let operation = if tls {
            Operation::ListenTls {
                host: "127.0.0.1".into(),
                port: i64::from(address.port()),
                certificate: cert.clone(),
                key: key.clone(),
                timeout: 5000,
            }
        } else {
            Operation::Listen("127.0.0.1".into(), i64::from(address.port()))
        };
        let mut job = Job::start(hosted.clone(), operation, None).unwrap();
        job.thread.take().unwrap().join().unwrap();
        // The native job succeeded and still owns a live listener despite having finished.
        assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 1);
        assert_eq!(
            std::net::TcpListener::bind(address).unwrap_err().kind(),
            io::ErrorKind::AddrInUse
        );
        drop(job);
        assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
        let _rebound = std::net::TcpListener::bind(address).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn abandoning_tls_configuration_joins_its_worker_and_releases_the_job() {
    use super::super::config_tests::Fixture;
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let fixture = Fixture::new();
    let path = fixture.path("certificate.fifo");
    nix::unistd::mkfifo(
        std::path::Path::new(&path),
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let job = Job::start(
        hosted.clone(),
        Operation::ListenTls {
            host: "127.0.0.1".into(),
            port: 0,
            certificate: path.clone(),
            key: "unused-key.pem".into(),
            timeout: 5000,
        },
        None,
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut writer = loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(writer) => break writer,
            Err(error)
                if error.raw_os_error() == Some(libc::ENXIO) && Instant::now() < deadline =>
            {
                std::thread::park_timeout(Duration::from_millis(1))
            }
            Err(error) => panic!("TLS worker did not start PEM I/O: {error}"),
        }
    };
    let stop = job.stop.clone();
    let (send, receive) = mpsc::channel();
    let dropping = std::thread::spawn(move || {
        drop(job);
        let _ = send.send(());
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while !stop.load(Ordering::Acquire) && Instant::now() < deadline {
        std::thread::yield_now();
    }
    let cancelled = stop.load(Ordering::Acquire);
    // Cancellation can win immediately after open, or while the parser is reading.
    // Release a pending read without requiring the native worker to start that read.
    if let Err(error) = writer.write_all(certificate.cert.pem().as_bytes()) {
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }
    drop(writer);
    let completed = receive.recv_timeout(Duration::from_secs(5));
    dropping.join().unwrap();
    assert!(cancelled, "abandonment must signal the native job");
    completed.unwrap();
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn saturated_listener_admission_does_not_bind_or_read_tls_files() {
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let permits = (0..MAX_PENDING_JOBS)
        .map(|_| Permit::acquire(hosted.clone()).unwrap())
        .collect::<Vec<_>>();
    for operation in [
        Operation::Listen("127.0.0.1".into(), 0),
        Operation::ListenTls {
            host: "127.0.0.1".into(),
            port: 0,
            certificate: "missing-certificate.pem".into(),
            key: "missing-key.pem".into(),
            timeout: 5000,
        },
    ] {
        match Job::start(hosted.clone(), operation, None) {
            Err(Failure::Native {
                kind: super::super::failure::NativeKind::IoFailure,
                code: None,
                detail,
            }) => assert!(detail.contains("pending per VM")),
            _ => panic!("listener admission must fail before OS work"),
        }
    }
    drop(permits);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
}

#[test]
fn abandoning_a_pending_read_joins_without_closing_its_connection() {
    use std::io::Write;
    let hosted = Arc::new(HostedState::new(crate::stdlib::Console::new(), Vec::new()));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let connection = hosted
        .network_connections
        .connect_tcp(
            "127.0.0.1",
            i64::from(listener.local_addr().expect("address").port()),
            5000,
        )
        .expect("connection");
    let (mut peer, _) = listener.accept().expect("peer");
    hosted
        .network_connections
        .set_timeout(connection, 5000)
        .expect("timeout");
    let job = Job::start(hosted.clone(), Operation::Read(connection, 1), None).expect("read");
    drop(job);
    assert_eq!(hosted.network_jobs.load(Ordering::Acquire), 0);
    peer.write_all(b"x").expect("send after abandonment");
    assert_eq!(
        hosted
            .network_connections
            .read(connection, 1)
            .expect("connection remains usable"),
        b"x"
    );
    hosted.network_connections.close(connection).expect("close");
}

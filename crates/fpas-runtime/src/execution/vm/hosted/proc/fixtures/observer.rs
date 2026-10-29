//! Host-owned sockets stay independent of VM teardown and prove child termination.

use std::io::Read;
use std::net::TcpListener;
use std::time::{Duration, Instant};

pub struct Observer(TcpListener);

impl Observer {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        Self(listener)
    }

    pub fn port(&self) -> String {
        self.0.local_addr().unwrap().port().to_string()
    }

    pub fn assert_closed(self, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(10);
        for _ in 0..count {
            let mut stream = loop {
                match self.0.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "child never connected to its host observer"
                        );
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("observer accept: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            assert_eq!(
                stream.read(&mut [0]).unwrap(),
                0,
                "child survived VM completion or abandonment"
            );
        }
    }
}

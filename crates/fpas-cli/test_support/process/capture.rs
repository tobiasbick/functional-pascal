//! Cancellable bounded readers; an escaped descendant cannot hold a Unix pipe forever.

use std::io::{self, Read};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::Duration;

const MAX_OUTPUT: usize = 8 * 1024 * 1024;

#[derive(Default)]
pub(super) struct Capture {
    pub(super) bytes: Arc<Mutex<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    reader: Option<JoinHandle<io::Result<()>>>,
}

impl Capture {
    pub(super) fn start(mut pipe: impl Read + Send + 'static) -> io::Result<Self> {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&bytes);
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let reader = std::thread::Builder::new()
            .name("fixture-output".into())
            .spawn(move || {
                let mut chunk = [0; 8192];
                loop {
                    let count = match pipe.read(&mut chunk) {
                        Ok(0) => return Ok(()),
                        Ok(count) => count,
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            if stopped.load(Ordering::Acquire) {
                                return Ok(());
                            }
                            std::thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    let mut output = captured
                        .lock()
                        .map_err(|_| io::Error::other("fixture output lock"))?;
                    if output.len() + count > MAX_OUTPUT {
                        return Err(io::Error::other("fixture output exceeded 8 MiB"));
                    }
                    output.extend_from_slice(&chunk[..count]);
                }
            })?;
        Ok(Self {
            bytes,
            stop,
            reader: Some(reader),
        })
    }

    pub(super) fn join(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(reader) = self.reader.take() {
            reader
                .join()
                .map_err(|_| io::Error::other("fixture output reader panicked"))??;
        }
        Ok(())
    }

    pub(super) fn take(&mut self) -> io::Result<Vec<u8>> {
        Ok(std::mem::take(
            &mut *self
                .bytes
                .lock()
                .map_err(|_| io::Error::other("fixture output lock"))?,
        ))
    }

    pub(super) fn text(&self) -> String {
        self.bytes
            .lock()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_else(|_| "<fixture output unavailable>".into())
    }
}

#[cfg(unix)]
pub(super) fn nonblocking(pipe: &impl std::os::fd::AsFd) -> io::Result<()> {
    use nix::fcntl::{FcntlArg, OFlag, fcntl};
    let flags = fcntl(pipe, FcntlArg::F_GETFL).map_err(io::Error::from)?;
    fcntl(
        pipe,
        FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK),
    )
    .map(|_| ())
    .map_err(io::Error::from)
}

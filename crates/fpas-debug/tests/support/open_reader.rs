//! Reader fixture that stays open after its scripted bytes are consumed.

use std::io::{self, Cursor, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

pub(crate) struct OpenReader {
    input: Cursor<Vec<u8>>,
    release: mpsc::Receiver<()>,
    dropped: Arc<AtomicBool>,
}

impl Read for OpenReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.input.read(buffer)?;
        if read != 0 {
            return Ok(read);
        }
        let _ = self.release.recv();
        Ok(0)
    }
}

impl Drop for OpenReader {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Release);
    }
}

pub(crate) fn open_reader(input: Vec<u8>) -> (OpenReader, Arc<AtomicBool>, mpsc::Sender<()>) {
    let (release, blocked) = mpsc::channel();
    let dropped = Arc::new(AtomicBool::new(false));
    (
        OpenReader {
            input: Cursor::new(input),
            release: blocked,
            dropped: Arc::clone(&dropped),
        },
        dropped,
        release,
    )
}

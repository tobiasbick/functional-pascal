//! Bounded draining of inherited benchmark pipes.

use std::io::{self, Read};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const OUTPUT_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

/// A background pipe drain and its completion channel.
pub(super) struct PipeReader {
    receiver: Receiver<io::Result<Vec<u8>>>,
    thread: JoinHandle<()>,
}

/// Starts an independent captured-output reader.
pub(super) fn read_pipe<R>(mut pipe: R) -> PipeReader
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    let thread = thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = pipe.read_to_end(&mut bytes).map(|_| bytes);
        let _ = sender.send(result);
    });
    PipeReader { receiver, thread }
}

/// Collects a reader with a bounded wait.
pub(super) fn finish_reader(reader: PipeReader, stream: &str) -> Result<Vec<u8>, String> {
    let result = match reader.receiver.recv_timeout(OUTPUT_DRAIN_TIMEOUT) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => {
            return Err(format!(
                "benchmark {stream} remained open after process termination"
            ));
        }
        Err(RecvTimeoutError::Disconnected) => {
            return Err(format!("benchmark {stream} reader panicked"));
        }
    };
    reader
        .thread
        .join()
        .map_err(|_| format!("benchmark {stream} reader panicked"))?;
    result.map_err(|error| format!("failed to read benchmark {stream}: {error}"))
}

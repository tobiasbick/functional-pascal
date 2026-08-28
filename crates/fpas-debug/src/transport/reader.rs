//! Owned request-reader thread with per-request lifecycle control.

use std::io::{self, BufReader, Read};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

enum ReaderAction {
    Continue,
    Stop,
}

/// Reader thread that waits for a server decision before reading another request.
pub(crate) struct ControlledReader<T> {
    receiver: Receiver<io::Result<T>>,
    actions: Sender<ReaderAction>,
    thread: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> ControlledReader<T> {
    /// Start a reader owned by the returned lifecycle controller.
    pub(crate) fn spawn<R, F>(reader: R, mut read_next: F) -> Self
    where
        R: Read + Send + 'static,
        F: FnMut(&mut BufReader<R>) -> io::Result<Option<T>> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let (actions, decisions) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            loop {
                match read_next(&mut reader) {
                    Ok(Some(request)) => {
                        if sender.send(Ok(request)).is_err()
                            || !matches!(decisions.recv(), Ok(ReaderAction::Continue))
                        {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        break;
                    }
                }
            }
        });
        Self {
            receiver,
            actions,
            thread: Some(thread),
        }
    }

    /// Wait up to `timeout` for the next complete request or reader failure.
    pub(crate) fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<io::Result<T>, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    /// Allow the reader to begin the next request.
    pub(crate) fn continue_reading(&self) {
        let _ = self.actions.send(ReaderAction::Continue);
    }

    /// Stop before the next read and join the owned thread.
    pub(crate) fn stop_and_join(self) -> io::Result<()> {
        let _ = self.actions.send(ReaderAction::Stop);
        self.join()
    }

    /// Join a reader that already reached EOF or failed.
    pub(crate) fn join(mut self) -> io::Result<()> {
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        thread
            .join()
            .map_err(|_| io::Error::other("debugger request reader thread panicked"))
    }
}

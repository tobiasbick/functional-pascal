use std::io::{self, Read, Write};
use std::pin::Pin;
use std::process::{ChildStderr, ChildStdin, ChildStdout};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::process;
use tokio::runtime::{Builder, Runtime};

pub(in crate::execution::vm::hosted::proc) struct PipeWriter {
    pipe: process::ChildStdin,
    runtime: Arc<Runtime>,
}

pub(in crate::execution::vm::hosted::proc) struct PipeReader {
    pipe: Reader,
    runtime: Arc<Runtime>,
}

enum Reader {
    Stdout(process::ChildStdout),
    Stderr(process::ChildStderr),
}

pub(super) fn runtime() -> io::Result<Arc<Runtime>> {
    // Standard Windows child pipes are synchronous. Tokio's safe from_std
    // adapters perform their I/O on bounded blocking workers; each engine probe
    // only polls their completion and never waits for a pipe operation.
    Builder::new_multi_thread()
        .worker_threads(1)
        .max_blocking_threads(3)
        .thread_name("fpas-proc-io")
        .build()
        .map(Arc::new)
}

pub(super) fn stdin(pipe: ChildStdin, runtime: Arc<Runtime>) -> io::Result<PipeWriter> {
    let pipe = {
        let _context = runtime.enter();
        process::ChildStdin::from_std(pipe)?
    };
    Ok(PipeWriter { pipe, runtime })
}

pub(super) fn stdout(pipe: ChildStdout, runtime: Arc<Runtime>) -> io::Result<PipeReader> {
    let pipe = {
        let _context = runtime.enter();
        Reader::Stdout(process::ChildStdout::from_std(pipe)?)
    };
    Ok(PipeReader { pipe, runtime })
}

pub(super) fn stderr(pipe: ChildStderr, runtime: Arc<Runtime>) -> io::Result<PipeReader> {
    let pipe = {
        let _context = runtime.enter();
        Reader::Stderr(process::ChildStderr::from_std(pipe)?)
    };
    Ok(PipeReader { pipe, runtime })
}

impl Read for PipeReader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let _context = self.runtime.enter();
        let mut context = Context::from_waker(Waker::noop());
        let mut buffer = ReadBuf::new(bytes);
        let result = match &mut self.pipe {
            Reader::Stdout(pipe) => Pin::new(pipe).poll_read(&mut context, &mut buffer),
            Reader::Stderr(pipe) => Pin::new(pipe).poll_read(&mut context, &mut buffer),
        };
        probe(result).map(|()| buffer.filled().len())
    }
}

impl Write for PipeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let _context = self.runtime.enter();
        let mut context = Context::from_waker(Waker::noop());
        probe(Pin::new(&mut self.pipe).poll_write(&mut context, bytes))
    }

    fn flush(&mut self) -> io::Result<()> {
        let _context = self.runtime.enter();
        let mut context = Context::from_waker(Waker::noop());
        probe(Pin::new(&mut self.pipe).poll_flush(&mut context))
    }
}

fn probe<T>(poll: Poll<io::Result<T>>) -> io::Result<T> {
    match poll {
        Poll::Ready(result) => result,
        Poll::Pending => Err(io::ErrorKind::WouldBlock.into()),
    }
}

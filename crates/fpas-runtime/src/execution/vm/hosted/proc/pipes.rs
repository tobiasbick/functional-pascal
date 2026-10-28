//! Fair bounded nonblocking stdio probes without detached pump threads.

use std::io::{self, Read, Write};

use super::failure::Kind;
use super::platform::{GroupChild, PipeReader, PipeWriter};

const CHUNK: usize = 8192;

pub(super) struct Pipes {
    input: Option<PipeWriter>,
    stdout: Option<PipeReader>,
    stderr: Option<PipeReader>,
    written: usize,
    maximum: usize,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    forward: Option<Forward>,
}

impl Pipes {
    pub(super) fn new(
        child: &mut GroupChild,
        maximum: usize,
        receiver: Option<crate::stdlib::ProgramStderr>,
    ) -> io::Result<Self> {
        Ok(Self {
            input: child.take_stdin()?,
            stdout: child.take_stdout()?,
            stderr: child.take_stderr()?,
            written: 0,
            maximum,
            stdout_bytes: Vec::new(),
            stderr_bytes: Vec::new(),
            forward: receiver.map(|receiver| Forward {
                receiver,
                pending: Vec::new(),
                maximum,
            }),
        })
    }

    pub(super) fn complete(&self) -> bool {
        self.input.is_none() && self.stdout.is_none() && self.stderr.is_none()
    }

    pub(super) fn poll(&mut self, input: &[u8]) -> Result<bool, Kind> {
        let mut progress = false;
        if let Some(writer) = &mut self.input {
            if self.written == input.len() {
                match writer.flush() {
                    Ok(()) => {
                        self.input = None;
                        progress = true;
                    }
                    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                        self.input = None;
                        progress = true;
                    }
                    Err(error) if pending(&error) => {}
                    Err(error) => return Err(Kind::Execution(error)),
                }
            } else {
                let end = input.len().min(self.written + CHUNK);
                match writer.write(&input[self.written..end]) {
                    Ok(0) => {
                        return Err(Kind::Execution(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "Process stdin accepted no bytes",
                        )));
                    }
                    Ok(count) => {
                        self.written += count;
                        progress = true;
                    }
                    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                        self.input = None;
                        progress = true;
                    }
                    Err(error) if pending(&error) => {}
                    Err(error) => return Err(Kind::Execution(error)),
                }
            }
        }
        progress |= read(
            &mut self.stdout,
            &mut self.stdout_bytes,
            self.maximum,
            "stdout bytes",
            None,
        )?;
        progress |= read(
            &mut self.stderr,
            &mut self.stderr_bytes,
            self.maximum,
            "stderr bytes",
            self.forward.as_mut(),
        )?;
        Ok(progress)
    }

    pub(super) fn close(&mut self) {
        self.input = None;
        self.stdout = None;
        self.stderr = None;
        if let Some(forward) = &mut self.forward {
            forward.finish();
        }
    }

    pub(super) fn drain_after_abort(&mut self) {
        self.input = None;
        let _ = read(
            &mut self.stdout,
            &mut self.stdout_bytes,
            self.maximum,
            "stdout bytes",
            None,
        );
        let _ = read(
            &mut self.stderr,
            &mut self.stderr_bytes,
            self.maximum,
            "stderr bytes",
            self.forward.as_mut(),
        );
    }

    pub(super) fn output(&self) -> (String, String) {
        (
            String::from_utf8_lossy(&self.stdout_bytes).into_owned(),
            String::from_utf8_lossy(&self.stderr_bytes).into_owned(),
        )
    }
}

fn read(
    pipe: &mut Option<PipeReader>,
    output: &mut Vec<u8>,
    maximum: usize,
    resource: &'static str,
    forward: Option<&mut Forward>,
) -> Result<bool, Kind> {
    let Some(reader) = pipe else {
        return Ok(false);
    };
    let mut bytes = [0; CHUNK];
    match reader.read(&mut bytes) {
        Ok(0) => {
            *pipe = None;
            if let Some(forward) = forward {
                forward.finish();
            }
            Ok(true)
        }
        Ok(count) => {
            if let Some(forward) = forward {
                forward.feed(&bytes[..count])?;
            } else {
                let keep = count.min(maximum - output.len());
                output.extend_from_slice(&bytes[..keep]);
                if keep < count {
                    return Err(Kind::Limit(resource, maximum));
                }
            }
            Ok(true)
        }
        Err(error) if pending(&error) => Ok(false),
        Err(error) => Err(Kind::Execution(error)),
    }
}

fn pending(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    )
}

struct Forward {
    receiver: crate::stdlib::ProgramStderr,
    pending: Vec<u8>,
    maximum: usize,
}

impl Forward {
    fn feed(&mut self, bytes: &[u8]) -> Result<(), Kind> {
        for byte in bytes {
            if *byte == b'\n' {
                self.emit();
            } else {
                if self.pending.len() == self.maximum {
                    return Err(Kind::Limit("stderr bytes", self.maximum));
                }
                self.pending.push(*byte);
            }
        }
        Ok(())
    }

    fn finish(&mut self) {
        if !self.pending.is_empty() {
            self.emit();
        }
    }

    fn emit(&mut self) {
        let text = String::from_utf8_lossy(&self.pending);
        (self.receiver)(text.trim_end_matches('\r'));
        self.pending.clear();
    }
}

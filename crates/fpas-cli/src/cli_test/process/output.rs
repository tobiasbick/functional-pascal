//! Bounded test output, including program stderr captured by a VM receiver.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::cli_output::{DiagnosticFormat, Reporter, program_stderr_receiver_with};

#[cfg(test)]
mod tests;

/// Captures program stderr for forwarding through the test reporter and worker protocol.
pub(in crate::cli_test) struct ProgramStderrBuffer {
    buffer: Arc<Mutex<CappedBuffer>>,
    truncation_record: Vec<u8>,
}

impl ProgramStderrBuffer {
    /// Creates a bounded buffer and the JSON-mode VM receiver that writes into it.
    pub(in crate::cli_test) fn new(
        format: DiagnosticFormat,
    ) -> (Self, Option<fpas_std::ProgramStderr>) {
        let truncation_record = fpas_diagnostics::render_program_stderr_json(
            "Program stderr was truncated at the 8 MiB capture limit.",
        )
        .map(|line| format!("{line}\n").into_bytes())
        .unwrap_or_default();
        // Reserve a complete truncation event so the stream stays bounded and valid JSONL.
        let limit = super::MAX_CAPTURED_OUTPUT.saturating_sub(truncation_record.len());
        let buffer = Arc::new(Mutex::new(CappedBuffer::new(limit)));
        let receiver_buffer = Arc::clone(&buffer);
        let receiver = program_stderr_receiver_with(format, move |bytes| {
            let mut buffer = receiver_buffer
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !buffer.overflowed() {
                let _ = buffer.write_all(bytes);
            }
        });
        (
            Self {
                buffer,
                truncation_record,
            },
            receiver,
        )
    }

    /// Forwards complete records and one truncation event, without changing the test outcome.
    pub(in crate::cli_test) fn forward(self, reporter: &mut Reporter<'_>) {
        let (rendered, overflowed) = {
            let mut buffer = self
                .buffer
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (std::mem::take(&mut buffer.bytes), buffer.overflowed())
        };
        reporter.forward_program_output(&rendered);
        if overflowed {
            reporter.forward_program_output(&self.truncation_record);
        }
    }
}

/// A byte sink that rejects writes beyond the isolated test output limit.
pub(super) struct CappedBuffer {
    bytes: Vec<u8>,
    limit: usize,
    overflowed: bool,
}

impl CappedBuffer {
    /// Creates an empty sink with the given maximum byte count.
    pub(super) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            overflowed: false,
        }
    }

    /// Reports whether a write exceeded the byte limit.
    pub(super) fn overflowed(&self) -> bool {
        self.overflowed
    }

    /// Returns the successfully captured bytes.
    pub(super) fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for CappedBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let remaining = self.limit.saturating_sub(self.bytes.len());
        if buffer.len() > remaining {
            self.overflowed = true;
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "isolated test output exceeded 8 MiB",
            ));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

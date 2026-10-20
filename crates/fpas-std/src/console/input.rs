use crate::error::{StdError, std_internal_error, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_INPUT_FAILURE;
use std::collections::VecDeque;
use std::io::{self, BufRead};

/// FIFO lines for `Std.Console.ReadLn` / line-buffered `Read` in tests (`Vm::push_readln_input`).
#[derive(Debug, Default, Clone)]
pub struct ReadLnQueue(VecDeque<String>);

impl ReadLnQueue {
    /// Creates an empty line queue.
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    /// Appends one line for a future buffered read.
    pub fn push_line(&mut self, line: &str) {
        self.0.push_back(line.to_string());
    }

    /// Returns whether no queued lines remain.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes the oldest line or returns a located input diagnostic.
    pub fn pop_line(&mut self, location: SourceLocation) -> Result<String, StdError> {
        self.0.pop_front().ok_or_else(|| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                "ReadLn: no input available (tests must push lines with Vm::push_readln_input)",
                "Queue input with Vm::push_readln_input before calling Std.Console.ReadText/ReadLn in tests.",
                location,
            )
        })
    }
}

/// Read one line from standard input, preserving its normalized line terminator.
/// A zero-byte read is EOF; a final unterminated line is still input.
pub fn read_line_from_stdin(location: SourceLocation) -> Result<Option<String>, StdError> {
    read_line_from_reader(&mut io::stdin().lock(), location)
}

pub(super) fn read_line_from_reader(
    reader: &mut impl BufRead,
    location: SourceLocation,
) -> Result<Option<String>, StdError> {
    let mut line = String::new();
    let count = reader.read_line(&mut line).map_err(|e| {
        std_runtime_error(
            RUNTIME_CONSOLE_INPUT_FAILURE,
            format!("ReadLn failed: {e}"),
            "Check stdin availability and try again.",
            location,
        )
    })?;
    if count == 0 {
        return Ok(None);
    }
    if line.ends_with("\r\n") {
        line.remove(line.len() - 2);
    }
    Ok(Some(line))
}

/// Line-buffered text stream shared by `Read` and `ReadLn` (classic Pascal `input`).
#[derive(Debug)]
pub struct TextInput {
    line_queue: ReadLnQueue,
    pending: VecDeque<char>,
    os_stdin: bool,
    eof: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a text stream backed by queued lines and process standard input.
    pub fn new() -> Self {
        Self {
            line_queue: ReadLnQueue::new(),
            pending: VecDeque::new(),
            os_stdin: true,
            eof: false,
        }
    }

    /// Construct a queue-only stream that never reads process stdin.
    pub fn without_os_stdin() -> Self {
        Self {
            os_stdin: false,
            ..Self::new()
        }
    }

    /// Appends one line for a future `Read` or `ReadLn` operation.
    pub fn push_line(&mut self, line: &str) {
        self.line_queue.push_line(line);
    }

    /// Treat further empty-queue reads as end of input.
    pub fn close_input(&mut self) {
        self.eof = true;
    }

    /// Drop unread queued lines and the current line buffer.
    pub fn clear_queued(&mut self) {
        self.line_queue = ReadLnQueue::new();
        self.pending.clear();
    }

    fn refill(&mut self, location: SourceLocation) -> Result<bool, StdError> {
        let line = if !self.line_queue.is_empty() {
            let mut line = self.line_queue.pop_line(location)?;
            line.push('\n');
            Some(line)
        } else if self.eof {
            None
        } else if self.os_stdin {
            read_line_from_stdin(location)?
        } else {
            return Err(std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                "ReadLn: no input available",
                "Queue a line before Read/ReadLn, or signal end of input.",
                location,
            ));
        };
        let Some(line) = line else {
            self.eof = true;
            return Ok(false);
        };
        self.pending.extend(line.chars());
        Ok(true)
    }

    /// One character from the buffered input stream (waits for a line if the buffer is empty).
    pub fn read_char(&mut self, location: SourceLocation) -> Result<Option<char>, StdError> {
        if self.pending.is_empty() && !self.refill(location)? {
            return Ok(None);
        }
        self.pending.pop_front().map(Some).ok_or_else(|| {
            std_internal_error(
                "Read: internal error (empty buffer after refill)",
                "This indicates a runtime bug. Please report this compiler/VM issue.",
                location,
            )
        })
    }

    /// Read until end-of-line (consumed); same logical stream as `read_char`.
    pub fn read_line(&mut self, location: SourceLocation) -> Result<Option<String>, StdError> {
        let mut out = String::new();
        loop {
            if self.pending.is_empty() && !self.refill(location)? {
                return Ok((!out.is_empty()).then_some(out));
            }
            let c = self.pending.pop_front().ok_or_else(|| {
                std_internal_error(
                    "ReadLn: internal error (empty buffer after refill)",
                    "This indicates a runtime bug. Please report this compiler/VM issue.",
                    location,
                )
            })?;
            if c == '\n' {
                return Ok(Some(out));
            }
            out.push(c);
        }
    }
}

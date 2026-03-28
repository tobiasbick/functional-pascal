use crate::error::{StdError, std_internal_error, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_CONSOLE_INPUT_FAILURE;
use std::collections::VecDeque;
use std::io::{self, BufRead};

/// FIFO lines for `Std.Console.ReadLn` / line-buffered `Read` in tests (`Vm::push_readln_input`).
#[derive(Debug, Default, Clone)]
pub struct ReadLnQueue(VecDeque<String>);

impl ReadLnQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push_line(&mut self, line: &str) {
        self.0.push_back(line.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop_line(&mut self, location: SourceLocation) -> Result<String, StdError> {
        self.0.pop_front().ok_or_else(|| {
            std_runtime_error(
                RUNTIME_CONSOLE_INPUT_FAILURE,
                "ReadLn: no input available (tests must push lines with Vm::push_readln_input)",
                "Queue input with Vm::push_readln_input before calling Std.Console.Read/ReadLn in tests.",
                location,
            )
        })
    }
}

/// Read one line from standard input (used when the test `ReadLnQueue` is empty).
pub fn read_line_from_stdin(location: SourceLocation) -> Result<String, StdError> {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).map_err(|e| {
        std_runtime_error(
            RUNTIME_CONSOLE_INPUT_FAILURE,
            format!("ReadLn failed: {e}"),
            "Check stdin availability and try again.",
            location,
        )
    })?;
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

/// Line-buffered text stream shared by `Read` and `ReadLn` (classic Pascal `input`).
#[derive(Debug, Default)]
pub struct TextInput {
    line_queue: ReadLnQueue,
    pending: VecDeque<char>,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_line(&mut self, line: &str) {
        self.line_queue.push_line(line);
    }

    fn refill(&mut self, location: SourceLocation) -> Result<(), StdError> {
        let line = if self.line_queue.is_empty() {
            read_line_from_stdin(location)?
        } else {
            self.line_queue.pop_line(location)?
        };
        for c in line.chars() {
            self.pending.push_back(c);
        }
        self.pending.push_back('\n');
        Ok(())
    }

    /// One character from the buffered input stream (waits for a line if the buffer is empty).
    pub fn read_char(&mut self, location: SourceLocation) -> Result<char, StdError> {
        if self.pending.is_empty() {
            self.refill(location)?;
        }
        self.pending.pop_front().ok_or_else(|| {
            std_internal_error(
                "Read: internal error (empty buffer after refill)",
                "This indicates a runtime bug. Please report this compiler/VM issue.",
                location,
            )
        })
    }

    /// Read until end-of-line (consumed); same logical stream as `read_char`.
    pub fn read_line(&mut self, location: SourceLocation) -> Result<String, StdError> {
        let mut out = String::new();
        loop {
            if self.pending.is_empty() {
                self.refill(location)?;
            }
            let c = self.pending.pop_front().ok_or_else(|| {
                std_internal_error(
                    "ReadLn: internal error (empty buffer after refill)",
                    "This indicates a runtime bug. Please report this compiler/VM issue.",
                    location,
                )
            })?;
            if c == '\n' {
                return Ok(out);
            }
            out.push(c);
        }
    }
}

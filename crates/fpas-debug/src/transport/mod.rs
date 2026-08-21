//! Bounded debugger transport I/O.

use std::io::{self, BufRead};

pub(crate) const MAX_DEBUGGER_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

/// Reads through one newline or EOF without growing beyond `max_bytes`.
pub(crate) fn read_bounded_line(
    reader: &mut impl BufRead,
    max_bytes: usize,
    limit_error: &'static str,
) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::with_capacity(max_bytes.min(8 * 1024));
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok((!line.is_empty()).then_some(line));
        }
        let chunk_bytes = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        let Some(next_len) = line.len().checked_add(chunk_bytes) else {
            return Err(invalid(limit_error));
        };
        if next_len > max_bytes {
            return Err(invalid(limit_error));
        }
        line.extend_from_slice(&available[..chunk_bytes]);
        reader.consume(chunk_bytes);
        if line.ends_with(b"\n") {
            return Ok(Some(line));
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

//! Standard DAP `Content-Length` message framing.

use std::io::{self, BufRead, Read, Write};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::Value;

use super::DapServer;

const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

/// Read one DAP message, returning `None` on a clean end of stream.
///
/// # Errors
///
/// Returns an actionable invalid-data error for malformed or truncated framing.
pub fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut header = String::new();
        let count = reader.read_line(&mut header)?;
        if count == 0 {
            return if content_length.is_none() {
                Ok(None)
            } else {
                Err(invalid("truncated DAP headers"))
            };
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        let Some((name, value)) = header.trim_end().split_once(':') else {
            return Err(invalid("malformed DAP header"));
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            let length = value
                .trim()
                .parse::<usize>()
                .map_err(|_| invalid("invalid DAP Content-Length"))?;
            if length > MAX_MESSAGE_BYTES {
                return Err(invalid("DAP message exceeds the 16 MiB limit"));
            }
            if content_length.replace(length).is_some() {
                return Err(invalid("duplicate DAP Content-Length"));
            }
        }
    }
    let length = content_length.ok_or_else(|| invalid("missing DAP Content-Length"))?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            invalid("truncated DAP message body")
        } else {
            error
        }
    })?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| invalid(format!("malformed DAP JSON: {error}")))
}

/// Write one compact DAP message with byte-accurate framing.
///
/// # Errors
///
/// Returns an I/O error when serialization or output fails.
pub fn write_message(writer: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Serve framed DAP requests until disconnect or end of input.
///
/// # Errors
///
/// Returns framing, transport, or serialization failures.
pub fn serve<R: Read + Send + 'static, W: Write>(
    reader: R,
    mut writer: W,
    mut server: DapServer,
) -> io::Result<()> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = io::BufReader::new(reader);
        loop {
            match read_message(&mut reader) {
                Ok(Some(message)) => {
                    if sender.send(Ok(message)).is_err() {
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
    loop {
        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(request)) => write_messages(&mut writer, server.handle(request))?,
            Ok(Err(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => write_messages(&mut writer, server.poll())?,
            Err(RecvTimeoutError::Disconnected) => {
                if server.is_running() {
                    write_messages(&mut writer, server.wait())?;
                }
                return Ok(());
            }
        }
        if server.is_terminated() {
            return Ok(());
        }
    }
}

fn write_messages(writer: &mut impl Write, messages: Vec<Value>) -> io::Result<()> {
    for message in messages {
        write_message(writer, &message)?;
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

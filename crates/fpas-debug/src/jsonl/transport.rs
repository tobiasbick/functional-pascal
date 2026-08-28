//! Concurrent line transport that keeps protocol stdout machine-readable.

use std::io::{self, BufReader, Read, Write};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use serde_json::Value;

use super::{JsonlServer, ServerStatus};
use crate::transport::{ControlledReader, MAX_DEBUGGER_MESSAGE_BYTES, read_bounded_line};

/// Serve JSONL requests until input closes or the debugger terminates.
///
/// # Errors
///
/// Returns an I/O error when request input or protocol output fails.
pub fn serve<R, W>(reader: R, mut writer: W, mut server: JsonlServer) -> io::Result<()>
where
    R: Read + Send + 'static,
    W: Write,
{
    let reader = ControlledReader::spawn(reader, read_request_line);

    loop {
        match reader.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(line)) => {
                let write_result = write_records(&mut writer, server.handle_line(&line));
                if write_result.is_err() || server.status() == ServerStatus::Terminated {
                    reader.stop_and_join()?;
                    write_result?;
                    return termination_result(&server);
                }
                reader.continue_reading();
            }
            Ok(Err(error)) => {
                reader.join()?;
                return Err(error);
            }
            Err(RecvTimeoutError::Timeout) => write_records(&mut writer, server.poll())?,
            Err(RecvTimeoutError::Disconnected) => {
                let mut write_result = Ok(());
                while server.status() == ServerStatus::Running {
                    write_result = write_records(&mut writer, server.wait());
                    if write_result.is_err() {
                        break;
                    }
                }
                reader.join()?;
                return write_result;
            }
        }
    }
}

/// Serve a finite command script and wait for each resume before reading the next request.
///
/// # Errors
///
/// Returns an I/O error when script input or protocol output fails.
pub fn serve_script<R: Read, W: Write>(
    reader: R,
    mut writer: W,
    mut server: JsonlServer,
) -> io::Result<()> {
    let mut reader = BufReader::new(reader);
    while let Some(line) = read_request_line(&mut reader)? {
        write_records(&mut writer, server.handle_line(&line))?;
        while server.status() == ServerStatus::Running {
            write_records(&mut writer, server.wait())?;
        }
        if server.status() == ServerStatus::Terminated {
            return termination_result(&server);
        }
    }
    while server.status() == ServerStatus::Running {
        write_records(&mut writer, server.wait())?;
    }
    Ok(())
}

fn read_request_line(reader: &mut impl std::io::BufRead) -> io::Result<Option<String>> {
    let Some(mut bytes) = read_bounded_line(
        reader,
        MAX_DEBUGGER_MESSAGE_BYTES,
        "JSONL request line exceeds the 16 MiB limit",
    )?
    else {
        return Ok(None);
    };
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    String::from_utf8(bytes).map(Some).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "JSONL request is not valid UTF-8",
        )
    })
}

fn termination_result(server: &JsonlServer) -> io::Result<()> {
    if server.terminated_fatally() {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fatal JSONL protocol error; inspect the emitted protocol_error record",
        ))
    } else {
        Ok(())
    }
}

fn write_records(writer: &mut impl Write, records: Vec<Value>) -> io::Result<()> {
    for record in records {
        let encoded = serde_json::to_vec(&record).map_err(io::Error::other)?;
        writer.write_all(&encoded)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

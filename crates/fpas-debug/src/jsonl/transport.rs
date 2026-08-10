//! Concurrent line transport that keeps protocol stdout machine-readable.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::Value;

use super::{JsonlServer, ServerStatus};

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
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if sender.send(Ok(line)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    });

    loop {
        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(line)) => write_records(&mut writer, server.handle_line(line.trim_end()))?,
            Ok(Err(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => write_records(&mut writer, server.poll())?,
            Err(RecvTimeoutError::Disconnected) => {
                if server.status() == ServerStatus::Running {
                    write_records(&mut writer, server.wait())?;
                }
                return Ok(());
            }
        }
        if server.status() == ServerStatus::Terminated {
            return Ok(());
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
    for line in BufReader::new(reader).lines() {
        write_records(&mut writer, server.handle_line(&line?))?;
        if server.status() == ServerStatus::Running {
            write_records(&mut writer, server.wait())?;
        }
        if server.status() == ServerStatus::Terminated {
            return Ok(());
        }
    }
    if server.status() == ServerStatus::Running {
        write_records(&mut writer, server.wait())?;
    }
    Ok(())
}

fn write_records(writer: &mut impl Write, records: Vec<Value>) -> io::Result<()> {
    for record in records {
        let encoded = serde_json::to_vec(&record).map_err(io::Error::other)?;
        writer.write_all(&encoded)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

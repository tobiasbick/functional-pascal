//! Shared helpers for `Std.Tui` session integration tests.

use fpas_bytecode::SourceLocation;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::console::Console;

pub(super) fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[derive(Clone)]
struct SharedBufferWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedBufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn console_with_shared_writer() -> (Console, Arc<Mutex<Vec<u8>>>) {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedBufferWriter {
        bytes: Arc::clone(&bytes),
    };
    (Console::with_writer(Box::new(writer)), bytes)
}

use std::io::{self, Write};

pub(super) struct CappedBuffer {
    bytes: Vec<u8>,
    limit: usize,
    overflowed: bool,
}

impl CappedBuffer {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            overflowed: false,
        }
    }

    pub(super) fn overflowed(&self) -> bool {
        self.overflowed
    }

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

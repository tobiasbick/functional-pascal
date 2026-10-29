//! Buffered byte position and recoverable ReadLine prefixes for one open file.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};

use super::failure::Failure;

pub(super) const MAX_BYTES: usize = 1_048_576;

pub(super) struct FileState {
    reader: BufReader<File>,
    pending: VecDeque<u8>,
    append: bool,
}

impl FileState {
    pub(super) fn new(reader: BufReader<File>, append: bool) -> Self {
        Self {
            reader,
            pending: VecDeque::new(),
            append,
        }
    }

    pub(super) fn read(&mut self, size: usize, path: &str) -> Result<Vec<u8>, Failure> {
        if !self.pending.is_empty() {
            return Ok(self.pending.drain(..size.min(self.pending.len())).collect());
        }
        let mut bytes = vec![0; size];
        let count = self
            .reader
            .read(&mut bytes)
            .map_err(|error| Failure::io(path, error))?;
        bytes.truncate(count);
        Ok(bytes)
    }

    pub(super) fn read_line(
        &mut self,
        path: &str,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Option<String>, Failure> {
        loop {
            if self.pending.back() == Some(&b'\n') {
                return self.finish_line(path, true).map(Some);
            }
            if self.pending.len() > MAX_BYTES
                && !(self.pending.len() == MAX_BYTES + 1 && self.pending.back() == Some(&b'\r'))
            {
                return Err(Failure::Limit(MAX_BYTES as i64));
            }
            if cancelled() {
                return Err(Failure::Cancelled);
            }
            let available = self
                .reader
                .fill_buf()
                .map_err(|error| Failure::io(path, error))?;
            if available.is_empty() {
                if self.pending.len() > MAX_BYTES {
                    return Err(Failure::Limit(MAX_BYTES as i64));
                }
                return if self.pending.is_empty() {
                    Ok(None)
                } else {
                    self.finish_line(path, false).map(Some)
                };
            }
            let until_lf = available
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(available.len(), |index| index + 1);
            let count = until_lf.min(MAX_BYTES + 2 - self.pending.len());
            self.pending.extend(&available[..count]);
            self.reader.consume(count);
        }
    }

    fn finish_line(&mut self, path: &str, terminated: bool) -> Result<String, Failure> {
        let ending = if terminated {
            1 + usize::from(self.pending.iter().rev().nth(1) == Some(&b'\r'))
        } else {
            0
        };
        if self.pending.len() - ending > MAX_BYTES {
            return Err(Failure::Limit(MAX_BYTES as i64));
        }
        let mut bytes: Vec<u8> = self.pending.drain(..).collect();
        if terminated {
            bytes.pop();
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
        }
        String::from_utf8(bytes).map_err(|error| Failure::Encoding(path.into(), error.to_string()))
    }

    fn position(&mut self) -> io::Result<u64> {
        self.reader
            .stream_position()?
            .checked_sub(self.pending.len() as u64)
            .ok_or_else(|| io::Error::other("Buffered file position is inconsistent"))
    }

    pub(super) fn tell(&mut self, path: &str) -> Result<i64, Failure> {
        let position = self.position().map_err(|error| Failure::io(path, error))?;
        i64::try_from(position).map_err(|_| Failure::Limit(i64::MAX))
    }

    pub(super) fn seek(&mut self, offset: i64, origin: i64, path: &str) -> Result<i64, Failure> {
        let base = match origin {
            0 => 0,
            1 => self.position().map_err(|error| Failure::io(path, error))? as i128,
            2 => self
                .reader
                .get_ref()
                .metadata()
                .map_err(|error| Failure::io(path, error))?
                .len() as i128,
            _ => return Err(Failure::argument("Origin", "Unknown seek origin")),
        };
        let target = base + i128::from(offset);
        if target < 0 {
            return Err(Failure::argument(
                "Offset",
                "Seek would move before the beginning",
            ));
        }
        if target > i128::from(i64::MAX) {
            return Err(Failure::Limit(i64::MAX));
        }
        self.reader
            .seek(SeekFrom::Start(target as u64))
            .map_err(|error| Failure::io(path, error))?;
        self.pending.clear();
        Ok(target as i64)
    }

    pub(super) fn write(&mut self, bytes: &[u8], path: &str) -> Result<usize, Failure> {
        self.prepare_write(path)?;
        write_some(self.reader.get_mut(), self.append, bytes)
            .map_err(|error| Failure::io(path, error))
    }

    fn prepare_write(&mut self, path: &str) -> Result<(), Failure> {
        let position = self.tell(path)? as u64;
        self.reader
            .seek(SeekFrom::Start(position))
            .map_err(|error| Failure::io(path, error))?;
        self.pending.clear();
        Ok(())
    }

    pub(super) fn size(&self, path: &str) -> Result<i64, Failure> {
        let length = self
            .reader
            .get_ref()
            .metadata()
            .map_err(|error| Failure::io(path, error))?
            .len();
        i64::try_from(length).map_err(|_| Failure::Limit(i64::MAX))
    }

    pub(super) fn set_length(&mut self, length: u64, path: &str) -> Result<(), Failure> {
        // Reconcile both lookahead buffers before resizing. File::set_len preserves
        // the OS position, including when truncation leaves it beyond the new EOF.
        self.prepare_write(path)?;
        self.reader
            .get_mut()
            .set_len(length)
            .map_err(|error| Failure::io(path, error))
    }

    pub(super) fn write_text(
        &mut self,
        text: &str,
        newline: bool,
        path: &str,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<(), Failure> {
        self.prepare_write(path)?;
        for mut chunk in text
            .as_bytes()
            .chunks(8192)
            .chain(newline.then_some(b"\n".as_slice()))
        {
            while !chunk.is_empty() {
                if cancelled() {
                    return Err(Failure::Cancelled);
                }
                match write_some(self.reader.get_mut(), self.append, chunk) {
                    Ok(0) => return Err(Failure::io(path, io::ErrorKind::WriteZero.into())),
                    Ok(count) => chunk = &chunk[count..],
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(Failure::io(path, error)),
                }
            }
        }
        Ok(())
    }

    pub(super) fn flush(&mut self, path: &str) -> Result<(), Failure> {
        self.reader
            .get_mut()
            .flush()
            .map_err(|error| Failure::io(path, error))
    }
}

fn write_some(file: &mut File, append: bool, bytes: &[u8]) -> io::Result<usize> {
    #[cfg(windows)]
    if append {
        use std::os::windows::fs::FileExt;
        // Windows' special EOF offset keeps each write append-only even though
        // the handle also has the FILE_WRITE_DATA right required by set_len.
        return file.seek_write(bytes, u64::MAX);
    }
    #[cfg(not(windows))]
    let _ = append; // Other platforms use the descriptor's native append flag.
    file.write(bytes)
}

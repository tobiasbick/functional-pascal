//! `Std.Io.Fs.AppendText` implementation.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// Append UTF-8 `text` to `path`, creating the file when it is missing.
pub(super) fn append_text(path: &Path, text: &str) -> io::Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(text.as_bytes())
}

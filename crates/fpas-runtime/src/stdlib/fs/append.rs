//! `Std.Io.Fs.AppendText` implementation.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use super::failure::Failure;
use std::fs::OpenOptions;
use std::path::Path;

/// Append UTF-8 `text` to `path`, creating the file when it is missing.
pub(super) fn append_text(
    path: &Path,
    text: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<(), Failure> {
    super::io::check(cancelled)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    super::io::write_all(&mut file, text.as_bytes(), cancelled)
}

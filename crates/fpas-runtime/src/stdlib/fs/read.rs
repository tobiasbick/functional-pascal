//! Bounded UTF-8 file reads for `Std.Io.Fs.ReadText`.

use super::failure::Failure;
use std::fs::File;

use crate::stdlib::limits::MAX_READ_TEXT_BYTES;

#[cfg(test)]
pub(super) fn read_text_limited(path: &str) -> Result<String, Failure> {
    read_text_controlled(path, &mut || false)
}

pub(super) fn read_text_controlled(
    path: &str,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<String, Failure> {
    super::io::check(cancelled)?;
    let file = File::open(path)?;
    if let Ok(metadata) = file.metadata()
        && metadata.len() > MAX_READ_TEXT_BYTES
    {
        return Err(Failure::LimitExceeded(
            "Std.Io.Fs.ReadText",
            MAX_READ_TEXT_BYTES as i64,
        ));
    }

    let bytes = super::io::read_prefix(file, MAX_READ_TEXT_BYTES.saturating_add(1), cancelled)?;
    let text =
        String::from_utf8(bytes).map_err(|error| Failure::InvalidEncoding(error.to_string()))?;
    if text.len() as u64 > MAX_READ_TEXT_BYTES {
        return Err(Failure::LimitExceeded(
            "Std.Io.Fs.ReadText",
            MAX_READ_TEXT_BYTES as i64,
        ));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::read_text_limited;

    fn unique_temp_path(name: &str) -> String {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("fpas-fs-read-{}-{id}-{name}", std::process::id()))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn read_text_returns_file_contents() {
        let path = unique_temp_path("read.txt");
        fs::write(&path, "hello fs").expect("write fixture");

        let text = read_text_limited(&path).expect("read fixture");

        assert_eq!(text, "hello fs");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_text_returns_error_for_missing_file() {
        let path = unique_temp_path("missing.txt");

        let result = read_text_limited(&path);

        assert!(result.is_err());
    }
}

//! Bounded UTF-8 file reads for `Std.Fs.ReadText`.

use std::fs::File;
use std::io::Read;

use crate::limits::MAX_READ_TEXT_BYTES;

pub(super) fn read_text_limited(path: &str) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    if let Ok(metadata) = file.metadata()
        && metadata.len() > MAX_READ_TEXT_BYTES
    {
        return Err(format!(
            "File `{path}` is larger than the maximum ReadText size of {MAX_READ_TEXT_BYTES} bytes.\n  help: Read smaller files, or split the input outside FPAS."
        ));
    }

    let mut limited = file.take(MAX_READ_TEXT_BYTES.saturating_add(1));
    let mut text = String::new();
    limited
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())?;
    if text.len() as u64 > MAX_READ_TEXT_BYTES {
        return Err(format!(
            "File `{path}` exceeds the maximum ReadText size of {MAX_READ_TEXT_BYTES} bytes.\n  help: Read smaller files, or split the input outside FPAS."
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

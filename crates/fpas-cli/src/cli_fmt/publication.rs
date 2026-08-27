//! Publishes formatted source text to its destination path.

use atomic_write_file::AtomicWriteFile;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Atomically replaces an existing source file with formatted text.
pub(super) fn write_source(path: &Path, text: &str) -> io::Result<()> {
    write_source_with(path, text, |destination, bytes| {
        destination.write_all(bytes)
    })
}

fn write_source_with(
    path: &Path,
    text: &str,
    write: impl FnOnce(&mut AtomicWriteFile, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let permissions = fs::metadata(path)?.permissions();
    let mut replacement = AtomicWriteFile::open(path)?;
    write(&mut replacement, text.as_bytes())?;
    replacement.as_file().set_permissions(permissions)?;
    replacement.commit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::create_temp_dir;

    #[test]
    fn partial_write_failure_preserves_existing_source() {
        let directory = create_temp_dir("fmt-atomic-partial-write");
        let path = directory.join("source.fpas");
        let original = "program Original;\nbegin\nend.\n";
        fs::write(&path, original).expect("source fixture must be written");

        let error = write_source_with(&path, "program Replacement;\nbegin\nend.\n", |file, _| {
            file.write_all(b"partial")?;
            Err(io::Error::other("injected partial write failure"))
        })
        .expect_err("injected write failure must be returned");
        let actual = fs::read_to_string(&path).expect("source must remain readable");
        fs::remove_dir_all(directory).expect("temp directory must be removed");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(actual, original);
    }

    #[cfg(unix)]
    #[test]
    fn replacement_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = create_temp_dir("fmt-atomic-permissions");
        let path = directory.join("source.fpas");
        fs::write(&path, "old").expect("source fixture must be written");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640))
            .expect("source permissions must be set");

        write_source(&path, "new").expect("replacement must succeed");
        let mode = fs::metadata(&path)
            .expect("replacement metadata must be readable")
            .permissions()
            .mode()
            & 0o777;
        fs::remove_dir_all(directory).expect("temp directory must be removed");

        assert_eq!(mode, 0o640);
    }
}

//! Same-directory atomic publication of complete application bundles.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

/// Validate and atomically publish an application executable.
///
/// The destination remains unchanged until the complete replacement has been
/// flushed and committed through one operating-system rename operation.
///
/// # Errors
///
/// Returns an error when the bundle is invalid, the staging file cannot be
/// written or marked executable, or the atomic commit fails.
pub fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    publish_with(path, bytes, AtomicWriteFile::commit)
}

fn publish_with(
    path: &Path,
    bytes: &[u8],
    commit: impl FnOnce(AtomicWriteFile) -> io::Result<()>,
) -> Result<(), String> {
    crate::decode(bytes).map_err(|error| format!("cannot publish invalid application: {error}"))?;
    let mut replacement = AtomicWriteFile::open(path)
        .map_err(|error| io_error("create temporary application for", path, error))?;
    replacement
        .write_all(bytes)
        .map_err(|error| io_error("write temporary application for", path, error))?;
    set_executable(replacement.as_file(), path)?;
    commit(replacement).map_err(|error| io_error("replace application", path, error))
}

#[cfg(unix)]
fn set_executable(file: &File, path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = file
        .metadata()
        .map_err(|error| io_error("read application metadata for", path, error))?
        .permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    file.set_permissions(permissions)
        .map_err(|error| io_error("mark application executable", path, error))
}

#[cfg(not(unix))]
fn set_executable(_file: &File, _path: &Path) -> Result<(), String> {
    Ok(())
}

fn io_error(operation: &str, path: &Path, error: io::Error) -> String {
    format!("failed to {operation} `{}`: {error}", path.display())
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "publication fault-injection fixtures use direct filesystem assertions"
    )]

    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use fpas_program::{Digest, ProgramIdentity, ProgramImage};

    use super::publish_with;

    fn temp_dir() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fpas-bundle-publication-unit-{}-{id}",
            std::process::id()
        ))
    }

    fn bundle() -> Vec<u8> {
        let (program, diagnostics) = fpas_parser::parse("program BundleFixture; begin end.");
        assert!(diagnostics.is_empty());
        let executable =
            fpas_compiler::compile_register_subset(&program).expect("fixture must compile");
        let image = ProgramImage::new(
            ProgramIdentity {
                compiler_version: "publication-test".to_string(),
                bytecode_version: fpas_bytecode::BYTECODE_VERSION,
                source_hash: Digest::of(b"source"),
                options_hash: Digest::of(b"options"),
                units: Vec::new(),
            },
            vec!["main.fpas".to_string()],
            executable,
        )
        .expect("valid image");
        let image = fpas_program::encode(&image).expect("encoded image");
        crate::encode(b"runner", &image, "demo").expect("valid bundle")
    }

    #[test]
    fn injected_commit_failure_preserves_destination_without_restore_phase() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("temporary directory");
        let destination = root.join("demo-app");
        fs::write(&destination, b"previous application").expect("previous application");

        let error = publish_with(&destination, &bundle(), |_replacement| {
            Err(std::io::Error::other("injected commit failure"))
        })
        .expect_err("publication must fail");

        assert!(error.contains("injected commit failure"));
        assert_eq!(
            fs::read(&destination).expect("preserved application"),
            b"previous application"
        );
        assert_eq!(
            fs::read_dir(&root).expect("transaction directory").count(),
            1,
            "failed staging must be discarded without backup or restore artifacts"
        );
        fs::remove_dir_all(root).ok();
    }
}

#![expect(
    clippy::expect_used,
    reason = "application publication fixtures use direct filesystem assertions"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_program::{Digest, ProgramIdentity, ProgramImage};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-bundle-publication-{}-{id}",
        std::process::id()
    ))
}

fn encoded_bundle(runner: &[u8]) -> Vec<u8> {
    let (program, diagnostics) = fpas_parser::parse("program BundleFixture; begin end.");
    assert!(diagnostics.is_empty());
    let executable = fpas_compiler::compile(&program).expect("fixture must compile");
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
    fpas_bundle::encode(runner, &image, "demo").expect("valid bundle")
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[test]
fn publication_creates_complete_new_application() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");
    let bundle = encoded_bundle(b"new runner");

    fpas_bundle::publish(&destination, &bundle).expect("publication");

    assert_eq!(fs::read(&destination).expect("application"), bundle);
    fs::remove_dir_all(root).ok();
}

#[test]
fn publication_atomically_replaces_existing_application() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");
    fs::write(&destination, encoded_bundle(b"old runner")).expect("old application");
    let replacement = encoded_bundle(b"new runner");

    fpas_bundle::publish(&destination, &replacement).expect("replacement");

    assert_eq!(
        fs::read(&destination).expect("replaced application"),
        replacement
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn invalid_bundle_preserves_existing_application() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");
    fs::write(&destination, b"previous application").expect("old application");

    let error = fpas_bundle::publish(&destination, b"invalid")
        .expect_err("invalid bundle must fail before staging");

    assert!(error.contains("cannot publish invalid application"));
    assert_eq!(
        fs::read(&destination).expect("preserved application"),
        b"previous application"
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn stale_legacy_candidates_do_not_block_or_get_deleted() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");
    let stale_temporary = append_suffix(&destination, ".123.1.tmp");
    let stale_backup = append_suffix(&destination, ".123.2.bak");
    fs::write(&stale_temporary, b"stale temporary").expect("stale temporary");
    fs::write(&stale_backup, b"stale backup").expect("stale backup");
    let replacement = encoded_bundle(b"new runner");

    fpas_bundle::publish(&destination, &replacement).expect("publication");

    assert_eq!(fs::read(&destination).expect("application"), replacement);
    assert_eq!(
        fs::read(&stale_temporary).expect("unowned temporary"),
        b"stale temporary"
    );
    assert_eq!(
        fs::read(&stale_backup).expect("unowned backup"),
        b"stale backup"
    );
    assert_eq!(
        fs::read_dir(&root).expect("publication directory").count(),
        3,
        "the committed staging file must not remain beside legacy files"
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn unremovable_legacy_backup_cannot_turn_commit_into_failure() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");
    let legacy_backup = append_suffix(&destination, ".bak");
    fs::create_dir(&legacy_backup).expect("unremovable legacy backup directory");
    let replacement = encoded_bundle(b"new runner");

    fpas_bundle::publish(&destination, &replacement).expect("committed publication");

    assert_eq!(fs::read(&destination).expect("application"), replacement);
    assert!(legacy_backup.is_dir());
    fs::remove_dir_all(root).ok();
}

#[cfg(unix)]
#[test]
fn publication_marks_application_executable() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let destination = root.join("demo-app");

    fpas_bundle::publish(&destination, &encoded_bundle(b"runner")).expect("publication");

    let mode = fs::metadata(&destination)
        .expect("application metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
    fs::remove_dir_all(root).ok();
}

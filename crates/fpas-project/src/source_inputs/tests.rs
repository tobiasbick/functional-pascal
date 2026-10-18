//! The input owner proves byte freezing, selection changes and aggregate admission.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-inputs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn request_freezes_exact_bytes_and_detects_content_and_membership_changes() {
    let f = Fixture::new();
    let source = f.0.join("a.fpas");
    let manifest = f.0.join("app.fpasprj");
    fs::write(&source, b"\xef\xbb\xbfprogram T;\rbegin end.\r\n").unwrap();
    fs::write(&manifest, b"[project]\nname='app'\n").unwrap();
    let inputs = SourceInputs::begin(|| false).unwrap();
    let original = read(&source).unwrap();
    read(&manifest).unwrap();
    assert_eq!(crate::expand_path_glob(&f.0, "*.fpas").unwrap().len(), 1);
    assert_eq!(read(&f.0.join("./a.fpas")).unwrap(), original);
    assert_eq!(inputs.files().len(), 2);
    assert!(inputs.verify().unwrap());
    fs::write(&source, b"program Changed; begin end.").unwrap();
    assert_eq!(read(&source).unwrap(), original);
    assert!(!inputs.verify().unwrap());
    fs::write(&source, &original).unwrap();
    fs::write(f.0.join("b.fpas"), b"unit Demo.B; end unit;").unwrap();
    assert_eq!(crate::expand_path_glob(&f.0, "*.fpas").unwrap().len(), 1);
    assert!(!inputs.verify().unwrap());
    fs::remove_file(f.0.join("b.fpas")).unwrap();
    fs::write(&manifest, b"changed").unwrap();
    assert!(!inputs.verify().unwrap());
    drop(inputs);
    assert_eq!(read(&manifest).unwrap(), b"changed");
}

#[test]
fn aggregate_budget_counts_canonical_bytes_once_and_cancellation_precedes_reads() {
    let f = Fixture::new();
    let bytes = vec![b' '; MAX_FILE_BYTES];
    let inputs = SourceInputs::begin(|| false).unwrap();
    for i in 0..8 {
        let path = f.0.join(format!("{i}.fpas"));
        fs::write(&path, &bytes).unwrap();
        read(&path).unwrap();
        read(&path).unwrap();
    }
    let extra = f.0.join("extra.fpas");
    fs::write(&extra, b"x").unwrap();
    assert_eq!(
        read(&extra).unwrap_err().kind(),
        io::ErrorKind::FileTooLarge
    );
    assert_eq!(inputs.reason(), Some("source_limit"));
    assert_eq!(inputs.files().len(), 8);
    drop(inputs);
    let inputs = SourceInputs::begin(|| true).unwrap();
    assert_eq!(read(&extra).unwrap_err().kind(), io::ErrorKind::Interrupted);
    assert_eq!(inputs.reason(), Some("query_cancelled"));
    assert!(inputs.files().is_empty());
}

#[test]
fn excluded_files_do_not_consume_snapshot_admission() {
    let f = Fixture::new();
    fs::create_dir_all(f.0.join("generated")).unwrap();
    fs::write(f.0.join("keep.fpas"), "unit Demo.Keep; end unit;").unwrap();
    for i in 0..4097 {
        fs::write(f.0.join(format!("generated/g{i}.fpas")), "ignored").unwrap();
    }
    let manifest = f.0.join("lib.fpasprj");
    fs::write(&manifest,"[project]\nname='lib'\nkind='library'\n[sources]\ninclude=['**/*.fpas']\nexclude=['generated/**/*.fpas']\n").unwrap();
    let inputs = SourceInputs::begin(|| false).unwrap();
    let selected = crate::load_project_source_paths(&manifest).unwrap();
    assert_eq!(selected, vec![f.0.join("keep.fpas")]);
    assert!(inputs.reason().is_none());
    assert!(inputs.verify().unwrap());
}

#[cfg(any(unix, windows))]
#[test]
fn linked_directories_are_selected_and_retargeting_never_replaces_the_first_identity() {
    let f = Fixture::new();
    for name in ["first", "second"] {
        fs::create_dir(f.0.join(name)).unwrap();
        fs::write(
            f.0.join(format!("{name}/core.fpas")),
            "unit Demo.Core; end unit;",
        )
        .unwrap();
    }
    fs::create_dir(f.0.join("src")).unwrap();
    let alias = f.0.join("src/linked");
    link(&f.0.join("first"), &alias);
    let inputs = SourceInputs::begin(|| false).unwrap();
    let files = crate::expand_path_glob(&f.0, "src/**/*.fpas").unwrap();
    assert_eq!(files, vec![alias.join("core.fpas")]);
    read(&files[0]).unwrap();
    unlink(&alias);
    link(&f.0.join("second"), &alias);
    assert!(read(&files[0]).is_err());
    assert_eq!(inputs.reason(), Some("source_changed"));
    assert!(!inputs.verify().unwrap());
}

#[cfg(windows)]
fn link(target: &Path, alias: &Path) {
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(alias.to_string_lossy().replace('/', "\\"))
        .arg(target.to_string_lossy().replace('/', "\\"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[cfg(windows)]
fn unlink(alias: &Path) {
    fs::remove_dir(alias).unwrap();
}
#[cfg(unix)]
fn link(target: &Path, alias: &Path) {
    std::os::unix::fs::symlink(target, alias).unwrap();
}
#[cfg(unix)]
fn unlink(alias: &Path) {
    fs::remove_file(alias).unwrap();
}

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) fn create_temp_dir(prefix: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let suffix = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "fpas-tests-{prefix}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temp directory must be created");
    dir
}

pub(crate) fn write_file(path: &Path) {
    fs::write(path, "").expect("test file must be created");
}

pub(crate) fn write_text(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directories must be created");
    }
    fs::write(path, text).expect("test file must be created");
}

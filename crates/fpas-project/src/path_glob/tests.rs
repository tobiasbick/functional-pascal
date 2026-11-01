//! Membership predictions must equal actual selection after creating the candidate.

use super::*;
use crate::source_inputs::SourceInputs;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-glob-candidate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("Src")).expect("source folder");
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

fn check(root: &Path, pattern: &str, exclude: &[&str], destination: &Path, expected: bool) {
    let exclude = exclude.iter().map(|s| (*s).into()).collect::<Vec<_>>();
    let _inputs = SourceInputs::begin(|| false).expect("bounded discovery");
    let predicted = would_select_path(root, pattern, &exclude, destination).expect("prediction");
    assert_eq!(predicted, expected, "{pattern} with {exclude:?}");
    fs::write(destination, "unit Candidate; end unit;").expect("create candidate");
    let physical = fs::canonicalize(destination).expect("physical candidate");
    let actual = expand_disk_selected(root, pattern, &exclude)
        .expect("actual membership")
        .iter()
        .any(|p| fs::canonicalize(p).is_ok_and(|p| p == physical));
    fs::remove_file(destination).expect("remove candidate");
    assert_eq!(
        predicted, actual,
        "predicted membership must match the owning walker"
    );
}

#[test]
fn candidate_prediction_preserves_lexical_exclusions_and_parent_relative_patterns() {
    let f = Fixture::new();
    let file = f.0.join("Src/new.fpas");
    check(&f.0, "**/*.fpas", &[], &file, true);
    check(&f.0, "**/*.fpas", &["Src/*.fpas"], &file, false);
    check(&f.0, "**/*.fpas", &["src/*.fpas"], &file, true);
    check(&f.0, "Src/new.fpas", &[], &file, true);
    check(&f.0.join("Src"), "../Src/*.fpas", &[], &file, true);
    check(&f.0, "*.fpas", &[], &file, false);
}

#[test]
fn linked_directory_candidates_obey_each_alias_exclusion() {
    let f = Fixture::new();
    let outside = Fixture::new();
    let target = outside.0.join("Src");
    let alias = f.0.join("Linked");
    #[cfg(windows)]
    let linked = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&alias)
        .arg(&target)
        .output()
        .expect("create directory junction");
    #[cfg(windows)]
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(&target, &alias);
    #[cfg(unix)]
    linked.expect("directory symlink for discovery fixture");
    let file = target.join("new.fpas");
    check(&f.0, "**/*.fpas", &[], &file, true);
    check(&f.0, "**/*.fpas", &["Linked/**"], &file, false);
    check(&f.0, "**/*.fpas", &["linked/**"], &file, true);
    #[cfg(windows)]
    fs::remove_dir(&alias).expect("remove junction");
    #[cfg(unix)]
    fs::remove_file(&alias).expect("remove symlink");
}

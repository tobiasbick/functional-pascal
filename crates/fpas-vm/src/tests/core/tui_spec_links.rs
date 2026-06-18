//! Tests for the project rule that TUI Rust sources link to the Pascal spec.
//!
//! **Documentation:** `docs/pascal/std/tui/app.md` (from the repository root).

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn tui_rust_sources_link_to_pascal_spec_docs() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("fpas-vm crate should live under crates/fpas-vm");
    let crates_dir = repo_root.join("crates");
    let mut tui_sources = Vec::new();
    collect_tui_rust_sources(&crates_dir, &mut tui_sources);

    let missing_links = tui_sources
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path).expect("Rust source should be readable");
            !source.contains("docs/pascal/")
        })
        .map(|path| {
            path.strip_prefix(repo_root)
                .expect("source should live under repository root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert!(
        missing_links.is_empty(),
        "TUI Rust sources must link to the canonical Pascal docs: {missing_links:?}"
    );
}

fn collect_tui_rust_sources(dir: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_tui_rust_sources(&path, output);
        } else if is_tui_rust_source(&path) {
            output.push(path);
        }
    }
}

fn is_tui_rust_source(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("rs")
        && path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.contains("tui"))
}

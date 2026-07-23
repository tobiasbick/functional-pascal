//! Tests for the project rule that TUI Rust sources link to the Pascal spec.
//!
//! **Documentation:** `docs/pascal/std/tui/README.md` (from the repository root).

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

#[test]
fn tui_rust_doc_links_point_to_existing_files() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("fpas-vm crate should live under crates/fpas-vm");
    let crates_dir = repo_root.join("crates");
    let mut rust_sources = Vec::new();
    collect_rust_sources(&crates_dir, &mut rust_sources);

    let mut missing_links = Vec::new();
    for path in rust_sources {
        let source = fs::read_to_string(&path).expect("Rust source should be readable");
        for (line_index, line) in source.lines().enumerate() {
            if !line.trim_start().starts_with("//") {
                continue;
            }
            for doc_path in tui_doc_paths(line) {
                let doc_file = doc_path.split_once('#').map_or(doc_path, |(file, _)| file);
                if !repo_root.join(doc_file).exists() {
                    let source_path = path
                        .strip_prefix(repo_root)
                        .expect("source should live under repository root")
                        .display();
                    missing_links.push(format!("{source_path}:{} -> {doc_path}", line_index + 1));
                }
            }
        }
    }

    assert!(
        missing_links.is_empty(),
        "TUI Rust doc links must point to existing files: {missing_links:?}"
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

fn collect_rust_sources(dir: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, output);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
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

fn tui_doc_paths(line: &str) -> Vec<&str> {
    const PREFIXES: [&str; 1] = ["docs/pascal/std/tui/"];

    let mut paths = Vec::new();
    for prefix in PREFIXES {
        let mut rest = line;
        while let Some(index) = rest.find(prefix) {
            let start = index;
            let after_prefix = index + prefix.len();
            let tail = &rest[after_prefix..];
            let tail_len = tail
                .find(|ch: char| !is_doc_path_char(ch))
                .unwrap_or(tail.len());
            let end = after_prefix + tail_len;
            paths.push(&rest[start..end]);
            rest = &rest[end..];
        }
    }
    paths
}

fn is_doc_path_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '#')
}

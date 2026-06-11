//! Format → re-parse round-trip tests (parser corpus and example programs).

mod common;

use std::path::{Path, PathBuf};

#[test]
fn parser_corpus_round_trip() {
    for (name, source) in common::corpus::SOURCES {
        common::assert_round_trip(name, source);
    }
}

#[test]
fn examples_pascal_round_trip() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pascal");
    walk_fpas_files(&root, &mut |path, source| {
        let label = path.strip_prefix(&root).unwrap_or(path).to_string_lossy();
        common::assert_round_trip(&label, source);
    });
}

fn walk_fpas_files(dir: &Path, visit: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_fpas_files(&path, visit);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "fpas") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        });
        visit(&path, &source);
    }
}

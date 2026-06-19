//! Format → re-parse round-trip tests (parser corpus and repository sources).

mod common;

use std::path::Path;

use common::walk::{repo_root, walk_fpas_files};

#[test]
fn parser_corpus_round_trip() {
    for (name, source) in common::corpus::SOURCES {
        common::assert_round_trip(name, source);
    }
}

#[test]
fn examples_tree_round_trip() {
    round_trip_tree("examples", &repo_root("examples"));
}

#[test]
fn tests_tree_round_trip() {
    round_trip_tree("tests", &repo_root("tests"));
}

#[test]
fn apps_tree_round_trip() {
    round_trip_tree("apps", &repo_root("apps"));
}

fn round_trip_tree(label: &str, root: &Path) {
    walk_fpas_files(root, &mut |path, source| {
        let relative = path
            .strip_prefix(repo_root("."))
            .unwrap_or(path)
            .to_string_lossy();
        common::assert_round_trip(&format!("{label}/{relative}"), source);
    });
}

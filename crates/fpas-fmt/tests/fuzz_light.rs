//! Lightweight format stability checks on a deterministic sample of repository sources.

#![allow(clippy::expect_used, clippy::panic)]

mod common;

use std::path::Path;

use common::walk::{REPO_SOURCE_ROOTS, collect_fpas_paths, repo_root};
use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

/// Stride for deterministic sampling across the full `.fpas` tree.
const SAMPLE_STRIDE: usize = 11;

#[test]
fn sampled_sources_format_and_reparse_without_panic() {
    let roots: Vec<_> = REPO_SOURCE_ROOTS
        .iter()
        .map(|root| repo_root(root))
        .collect();
    let paths = collect_fpas_paths(&roots);
    assert!(
        paths.len() >= SAMPLE_STRIDE,
        "expected at least {SAMPLE_STRIDE} .fpas files under {:?}",
        REPO_SOURCE_ROOTS
    );

    let mut checked = 0usize;
    for (index, path) in paths.iter().enumerate() {
        if index % SAMPLE_STRIDE != 0 {
            continue;
        }
        format_and_reparse(path);
        checked += 1;
    }

    assert!(
        checked >= 20,
        "sample too small: checked {checked} of {} files",
        paths.len()
    );
}

fn format_and_reparse(path: &Path) {
    let label = path.to_string_lossy();
    let source = std::fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("{label}: failed to read source: {err}");
    });

    let (unit, errors) = parse_compilation_unit(&source);
    assert!(
        errors.is_empty(),
        "{label}: source must parse before format: {errors:?}"
    );

    let formatted = format_source(&source, &unit).expect("matching source and AST");
    let (_, errors) = parse_compilation_unit(&formatted);
    assert!(
        errors.is_empty(),
        "{label}: formatted output must re-parse: {errors:?}\n--- formatted ---\n{formatted}"
    );

    let formatted_again = format_source(&formatted, &parse_compilation_unit(&formatted).0)
        .expect("matching formatted source and AST");
    assert_eq!(
        formatted, formatted_again,
        "{label}: format must be idempotent"
    );
}

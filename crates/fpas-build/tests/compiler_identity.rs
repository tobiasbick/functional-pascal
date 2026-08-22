//! Integration tests for compiler identity emitted by the build script.

#![allow(
    clippy::expect_used,
    reason = "the build-script fixture has a required static declaration"
)]

const COMPILER_IDENTITY_SOURCE: &str = include_str!("../build/compiler_identity.rs");

#[test]
fn compiler_identity_lists_every_build_relevant_workspace_crate() {
    let expected = [
        "fpas-build",
        "fpas-bytecode",
        "fpas-compiler",
        "fpas-lexer",
        "fpas-linker",
        "fpas-parser",
        "fpas-program",
        "fpas-project",
        "fpas-sema",
        "fpas-std",
        "fpas-unit",
    ];

    let (_, after_declaration) = COMPILER_IDENTITY_SOURCE
        .split_once("const COMPILER_CRATES: &[&str] = &[")
        .expect("compiler identity crate declaration");
    let (list, _) = after_declaration
        .split_once("];")
        .expect("compiler identity crate list terminator");
    let actual = list
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|line| line.strip_suffix("\","))
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

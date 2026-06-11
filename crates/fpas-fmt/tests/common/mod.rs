//! Shared helpers for integration tests.

pub mod corpus;

use fpas_fmt::format_compilation_unit;
use fpas_parser::parse_compilation_unit;

/// Parses `source`, formats it, and asserts the formatted text parses without errors.
pub fn assert_round_trip(name: &str, source: &str) {
    let (unit, source_errors) = parse_compilation_unit(source);
    assert!(
        source_errors.is_empty(),
        "{name}: source must parse: {source_errors:?}"
    );
    let formatted = format_compilation_unit(&unit);
    let (_, errors) = parse_compilation_unit(&formatted);
    assert!(
        errors.is_empty(),
        "{name}: formatted output must re-parse: {errors:?}\n--- formatted ---\n{formatted}"
    );
}

/// Parses `source`, formats it, and compares to `expected`.
pub fn assert_golden(name: &str, source: &str, expected: &str) {
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{name}: {errors:?}");
    let formatted = format_compilation_unit(&unit);
    assert_eq!(formatted, normalize_newlines(expected), "{name}");
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

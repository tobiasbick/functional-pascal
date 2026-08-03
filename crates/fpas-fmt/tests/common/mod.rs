//! Shared helpers for integration tests.

#![allow(dead_code)]

pub mod corpus;
pub mod walk;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

/// Parses `source`, formats it, and asserts the formatted text parses without errors.
pub fn assert_round_trip(name: &str, source: &str) {
    let (unit, source_errors) = parse_compilation_unit(source);
    assert!(
        source_errors.is_empty(),
        "{name}: source must parse: {source_errors:?}"
    );
    let formatted = format_source(source, &unit).expect("matching source and AST");
    assert_eq!(
        normalized_comments(source),
        normalized_comments(&formatted),
        "{name}: formatting must preserve every comment"
    );
    let (_, errors) = parse_compilation_unit(&formatted);
    assert!(
        errors.is_empty(),
        "{name}: formatted output must re-parse: {errors:?}\n--- formatted ---\n{formatted}"
    );

    let (unit_again, _) = parse_compilation_unit(&formatted);
    let formatted_again =
        format_source(&formatted, &unit_again).expect("matching formatted source and AST");
    assert_eq!(
        formatted, formatted_again,
        "{name}: format must be idempotent\n--- first ---\n{formatted}\n--- second ---\n{formatted_again}"
    );
}

/// Parses `source`, formats it, and compares to `expected`.
pub fn assert_golden(name: &str, source: &str, expected: &str) {
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{name}: {errors:?}");
    let formatted = format_source(source, &unit).expect("matching source and AST");
    assert_eq!(formatted, normalize_newlines(expected), "{name}");
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn normalized_comments(source: &str) -> Vec<String> {
    fpas_lexer::collect_comments(source)
        .iter()
        .map(|comment| {
            comment
                .text(source)
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .trim_end()
                .to_string()
        })
        .collect()
}

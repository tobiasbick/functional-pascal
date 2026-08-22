//! Regression tests for formatting literal values without changing their meaning.

#![allow(clippy::expect_used)]

mod common;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

#[test]
fn control_characters_emit_pascal_character_codes_idempotently() {
    let source = "program T; begin var S: string := 'A'#0#9#13#10'B''é' end.";
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "source must parse: {diagnostics:?}");

    let formatted = format_source(source, &unit).expect("format control-character source");
    assert!(formatted.contains("'A'#0#9#13#10'B''é'"), "{formatted}");
    assert!(
        formatted.bytes().all(|byte| byte == b'\n' || byte >= b' '),
        "formatted source contains a raw control byte: {formatted:?}"
    );
    common::assert_round_trip("control-character literal", &formatted);
}

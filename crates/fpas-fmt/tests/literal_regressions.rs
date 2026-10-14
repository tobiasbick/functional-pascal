//! Regression tests for formatting literal values without changing their meaning.

#![allow(clippy::expect_used)]

mod common;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

#[test]
fn control_characters_emit_pascal_character_codes_idempotently() {
    let source = "program T; begin const S: string := 'A'#0#9#13#10'B''é'; end.";
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

#[test]
fn unicode_character_codes_round_trip_without_changing_decoded_values() {
    let source = "program T; begin const S := #0#255#256#8_364#55295#57344#128512#1_114_111#13#10'''X'; end.";
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let formatted = format_source(source, &unit).expect("format Unicode scalar literals");
    let (tokens, errors) = fpas_lexer::lex(&formatted);
    assert!(errors.is_empty(), "{errors:?}");
    let values = tokens
        .into_iter()
        .filter_map(|token| match token.token {
            fpas_lexer::Token::Str(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(values, ["\0\u{ff}Ā€\u{d7ff}\u{e000}😀\u{10ffff}\r\n'X"]);
    common::assert_round_trip("Unicode scalar literals", &formatted);
}

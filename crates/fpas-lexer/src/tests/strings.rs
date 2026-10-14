use super::toks;
use crate::Token;

// ── Simple Strings ──────────────────────────────────────────────

#[test]
fn simple_string() {
    assert_eq!(toks("'hello'"), vec![Token::Str("hello".into())]);
}

#[test]
fn empty_string() {
    assert_eq!(toks("''"), vec![Token::Str("".into())]);
}

#[test]
fn single_char_string() {
    assert_eq!(toks("'A'"), vec![Token::Str("A".into())]);
}

// ── Escaped Apostrophes ─────────────────────────────────────────

#[test]
fn escaped_apostrophe() {
    assert_eq!(toks("'It''s'"), vec![Token::Str("It's".into())]);
}

#[test]
fn single_apostrophe_value() {
    assert_eq!(toks("''''"), vec![Token::Str("'".into())]);
}

#[test]
fn multiple_escapes() {
    assert_eq!(toks("'a''b''c'"), vec![Token::Str("a'b'c".into())]);
}

#[test]
fn double_apostrophe_at_end() {
    assert_eq!(toks("'hello'''"), vec![Token::Str("hello'".into())]);
}

// ── Multi-line Strings ──────────────────────────────────────────

#[test]
fn multi_line() {
    assert_eq!(
        toks("'line1\nline2'"),
        vec![Token::Str("line1\nline2".into())]
    );
}

#[test]
fn multi_line_crlf() {
    assert_eq!(
        toks("'line1\r\nline2'"),
        vec![Token::Str("line1\r\nline2".into())]
    );
}

// ── Char Codes ──────────────────────────────────────────────────

#[test]
fn char_code_letter() {
    assert_eq!(toks("#65"), vec![Token::Str("A".into())]);
}

#[test]
fn char_code_tab() {
    assert_eq!(toks("#9"), vec![Token::Str("\t".into())]);
}

#[test]
fn char_code_null() {
    assert_eq!(toks("#0"), vec![Token::Str("\0".into())]);
}

#[test]
fn char_code_latin1() {
    assert_eq!(toks("#255"), vec![Token::Str("\u{FF}".into())]);
}

#[test]
fn char_code_space() {
    assert_eq!(toks("#32"), vec![Token::Str(" ".into())]);
}

// ── Char Code Concatenation ─────────────────────────────────────

#[test]
fn char_code_crlf() {
    assert_eq!(toks("#13#10"), vec![Token::Str("\r\n".into())]);
}

#[test]
fn string_with_char_codes() {
    assert_eq!(
        toks("'Hello'#13#10'World'"),
        vec![Token::Str("Hello\r\nWorld".into())]
    );
}

#[test]
fn char_code_between_strings() {
    // ASCII 32 = space
    assert_eq!(toks("'A'#32'B'"), vec![Token::Str("A B".into())]);
}

#[test]
fn char_code_then_string() {
    // ASCII 72 = 'H'
    assert_eq!(toks("#72'ello'"), vec![Token::Str("Hello".into())]);
}

#[test]
fn string_then_char_code() {
    // ASCII 33 = '!'
    assert_eq!(toks("'Hi'#33"), vec![Token::Str("Hi!".into())]);
}

#[test]
fn empty_string_with_char_code() {
    assert_eq!(toks("''#65"), vec![Token::Str("A".into())]);
}

// ── Separate Strings (whitespace breaks concatenation) ──────────

#[test]
fn whitespace_breaks_concatenation() {
    assert_eq!(
        toks("'hello' 'world'"),
        vec![Token::Str("hello".into()), Token::Str("world".into())]
    );
}

#[test]
fn space_between_char_codes() {
    assert_eq!(
        toks("#65 #66"),
        vec![Token::Str("A".into()), Token::Str("B".into())]
    );
}

// ── Edge Cases (language/basics) ───────────────────────────────────

#[test]
fn char_codes_only() {
    // Multiple char codes concatenated without any quoted string
    assert_eq!(toks("#65#66#67"), vec![Token::Str("ABC".into())]);
}

#[test]
fn char_code_boundary_values() {
    for (source, expected) in [
        ("#0", "\0"),
        ("#255", "\u{ff}"),
        ("#256", "Ā"),
        ("#8364", "€"),
        ("#55295", "\u{d7ff}"),
        ("#57344", "\u{e000}"),
        ("#128512", "😀"),
        ("#1114111", "\u{10ffff}"),
        ("#8_364", "€"),
        ("#1_114_111", "\u{10ffff}"),
    ] {
        assert_eq!(toks(source), vec![Token::Str(expected.into())], "{source}");
    }
}

#[test]
fn legacy_character_codes_keep_their_scalar_values() {
    for code in 0_u8..=255 {
        assert_eq!(
            toks(&format!("#{code}")),
            vec![Token::Str(char::from(code).to_string())],
            "#{code}"
        );
    }
}

#[test]
fn unicode_character_codes_concatenate_with_exact_source_positions() {
    let source = " \r\n'😀'#8_364#128512'Z';";
    let (tokens, _, errors) = crate::lex_with_source_id(source, 17);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(tokens[0].token, Token::Str("😀€😀Z".into()));
    assert_eq!(tokens[0].span.text(source), Some("'😀'#8_364#128512'Z'"));
    assert_eq!(tokens[0].span.source_id, 17);
    assert_eq!(tokens[0].span.offset, 3);
    assert_eq!((tokens[0].span.line, tokens[0].span.column), (2, 1));
    assert_eq!(tokens[0].end.offset, 25);
    assert_eq!((tokens[0].end.line, tokens[0].end.column), (2, 20));
    assert_eq!(tokens[1].token, Token::Semicolon);
    assert_eq!(tokens[1].span.offset, 25);
}

#[test]
fn string_with_only_escaped_apostrophes() {
    // '''''' = two doubled apostrophes inside quotes = "''"
    assert_eq!(toks("''''''"), vec![Token::Str("''".into())]);
}

#[test]
fn empty_string_between_char_codes() {
    // #65''#66 = char code, empty string, char code — all concatenated
    assert_eq!(toks("#65''#66"), vec![Token::Str("AB".into())]);
}

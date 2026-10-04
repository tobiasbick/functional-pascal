use super::parse_with_errors;

const KEYWORDS: &[&str] = &[
    "program",
    "unit",
    "uses",
    "const",
    "var",
    "function",
    "procedure",
    "begin",
    "end",
    "return",
    "if",
    "then",
    "else",
    "case",
    "of",
    "for",
    "to",
    "downto",
    "in",
    "do",
    "while",
    "repeat",
    "until",
    "and",
    "or",
    "not",
    "xor",
    "div",
    "mod",
    "true",
    "false",
    "type",
    "record",
    "enum",
    "array",
    "channel",
    "panic",
    "break",
    "continue",
    "public",
    "result",
    "option",
    "ok",
    "error",
    "some",
    "none",
    "try",
    "go",
    "dict",
    "with",
    "comparable",
    "numeric",
    "printable",
];

#[test]
fn every_keyword_is_rejected_as_a_member_name() {
    for keyword in KEYWORDS {
        let source = format!("program T; begin Value.{keyword}(); end program;");
        let (_, errors) = parse_with_errors(&source);
        assert!(!errors.is_empty(), "`{keyword}` was accepted after `.`");
    }
}

#[test]
fn unknown_generic_constraint_is_rejected_by_the_parser() {
    let (_, errors) = parse_with_errors(
        "program T; function Identity of (T: Nonexistent)(Value: T): T; begin return Value end; begin end.",
    );
    assert!(!errors.is_empty());
}

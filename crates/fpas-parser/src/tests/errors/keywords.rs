use super::parse_with_errors;

const KEYWORDS: &[&str] = &[
    "program",
    "unit",
    "uses",
    "const",
    "var",
    "mutable",
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
    "shl",
    "shr",
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
    "static",
    "property",
    "event",
    "read",
    "write",
    "comparable",
    "numeric",
    "printable",
    "self",
    "nil",
];

#[test]
fn every_keyword_is_rejected_as_a_member_name() {
    for keyword in KEYWORDS {
        let source = format!("program T; begin Value.{keyword}() end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(!errors.is_empty(), "`{keyword}` was accepted after `.`");
    }
}

#[test]
fn self_is_rejected_as_an_ordinary_parameter_name() {
    let (_, errors) = parse_with_errors(
        "program T; function Identity(Self: integer): integer; begin return Self end; begin end.",
    );
    assert!(!errors.is_empty());
}

#[test]
fn self_is_rejected_as_a_static_method_parameter_name() {
    for declaration in [
        "static function Create(Self: Point): Point; begin return Self end;",
        "static procedure Reset(Self: Point); begin end;",
    ] {
        let source = format!("program T; type Point = record {declaration} end; begin end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(!errors.is_empty(), "static declaration accepted `Self`");
    }
}

#[test]
fn unknown_generic_constraint_is_rejected_by_the_parser() {
    let (_, errors) = parse_with_errors(
        "program T; function Identity<T: Nonexistent>(Value: T): T; begin return Value end; begin end.",
    );
    assert!(!errors.is_empty());
}

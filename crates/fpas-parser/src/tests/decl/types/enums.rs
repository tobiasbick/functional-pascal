use super::*;
use fpas_diagnostics::codes::{PARSE_EMPTY_ENUM_FIELD_LIST, PARSE_TRAILING_ENUM_FIELD_SEPARATOR};

#[test]
fn enum_type() {
    let p = parse_ok("program T; type Color = enum Red; Green; Blue; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 3);
                assert_eq!(e.members[0].name, "Red");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_with_values() {
    let p = parse_ok("program T; type Suit = enum Hearts = 1; Diamonds = 2; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members[0].value, Some(1));
                assert_eq!(e.members[1].value, Some(2));
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_member_non_integer_value_uses_parser_code() {
    let (_, errors) =
        parse_with_errors("program T; type Suit = enum Hearts = true; end; begin end.");
    let error = errors
        .iter()
        .find_map(|diagnostic| match diagnostic {
            ParseDiagnostic::Parser(error) => Some(error),
            ParseDiagnostic::Lexer(_) => None,
        })
        .expect("expected parser diagnostic");
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("integer literals")),
        "expected explicit enum-value help text"
    );
}

#[test]
fn enum_data_single_field() {
    let p = parse_ok("program T; type Wrapper = enum Val(X: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 1);
                assert_eq!(e.members[0].name, "Val");
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[0].fields[0].name, "X");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_multiple_fields() {
    let p = parse_ok(
        "program T; type Shape = enum Circle(Radius: real); Rect(W: real; H: real); end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[1].fields.len(), 2);
                assert_eq!(e.members[1].fields[0].name, "W");
                assert_eq!(e.members[1].fields[1].name, "H");
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_mixed_simple_and_data_variants() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert!(e.members[0].fields.is_empty());
                assert_eq!(e.members[1].fields.len(), 1);
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_fieldless_has_no_backing_value() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert!(e.members[0].value.is_none());
                assert!(e.members[1].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_variant_cannot_mix_fields_with_backing_value() {
    let (_, errors) =
        parse_with_errors("program T; type Shape = enum Circle(Radius: real) = 1; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parser error when mixing enum fields with a backing value"
    );
}

#[test]
fn enum_empty_field_list_is_rejected() {
    let (_, errors) = parse_with_errors("program T; type Token = enum Eof(); end; begin end.");
    let diagnostic = errors.iter().find_map(|error| match error {
        ParseDiagnostic::Parser(diagnostic) if diagnostic.code == PARSE_EMPTY_ENUM_FIELD_LIST => {
            Some(diagnostic)
        }
        _ => None,
    });

    let diagnostic = diagnostic
        .unwrap_or_else(|| panic!("expected empty enum field-list diagnostic, got: {errors:#?}"));
    assert_eq!(
        diagnostic.help.as_deref(),
        Some(
            "Write `Variant;` for a fieldless variant or add a field such as `Variant(Value: integer);`."
        )
    );
}

#[test]
fn enum_empty_field_list_before_backing_value_is_rejected() {
    let (_, errors) = parse_with_errors("program T; type Token = enum Eof() = 5; end; begin end.");
    assert!(
        errors.iter().any(|error| matches!(
            error,
            ParseDiagnostic::Parser(diagnostic)
                if diagnostic.code == PARSE_EMPTY_ENUM_FIELD_LIST
        )),
        "{errors:#?}"
    );
}

#[test]
fn enum_trailing_field_separator_is_rejected() {
    let (_, errors) = parse_with_errors(
        "program T; type Shape = enum Rectangle(Width: real; Height: real;); end; begin end.",
    );
    let diagnostic = errors.iter().find_map(|error| match error {
        ParseDiagnostic::Parser(diagnostic)
            if diagnostic.code == PARSE_TRAILING_ENUM_FIELD_SEPARATOR =>
        {
            Some(diagnostic)
        }
        _ => None,
    });

    let diagnostic = diagnostic.unwrap_or_else(|| {
        panic!("expected trailing enum field separator diagnostic, got: {errors:#?}")
    });
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("Remove the trailing separator: write `Variant(First: integer; Second: integer);`.")
    );
}

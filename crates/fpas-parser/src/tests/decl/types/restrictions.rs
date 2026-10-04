use super::*;

#[test]
fn generic_record_type_params_not_allowed() {
    let (_, errors) =
        parse_with_errors("program T; type Box of (T) = record Value: integer; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic type definition"
    );
}

#[test]
fn generic_enum_type_params_not_allowed() {
    let (_, errors) =
        parse_with_errors("program T; type Maybe of (T) = enum Just; Nothing; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic enum definition"
    );
}

#[test]
fn generic_type_alias_of_syntax_not_allowed() {
    let (_, errors) = parse_with_errors(
        r#"program T; type Foo = integer; const X: Foo of integer := 0; begin end."#,
    );
    assert!(
        !errors.is_empty(),
        "expected parse error for generic type argument syntax"
    );
}

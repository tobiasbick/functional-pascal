use super::*;

#[test]
fn angle_bracket_record_type_params_are_rejected() {
    let (_, errors) =
        parse_with_errors("program T; type Box<T> = record Value: integer; end record; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic type definition"
    );
}

#[test]
fn angle_bracket_enum_type_params_are_rejected() {
    let (_, errors) =
        parse_with_errors("program T; type Maybe<T> = enum Just; Nothing; end enum; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic enum definition"
    );
}

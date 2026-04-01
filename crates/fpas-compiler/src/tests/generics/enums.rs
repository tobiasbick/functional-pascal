use super::*;

// Generic enum type definitions produce a parse error.

#[test]
fn generic_enum_type_params_produce_parse_error() {
    parse_fails(
        "\
program T;
type Maybe<T> = enum Just; Nothing; end;
begin end.",
    );
}

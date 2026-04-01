use super::*;

// Generic type definitions on records produce a parse error.

#[test]
fn generic_record_type_params_produce_parse_error() {
    parse_fails(
        "\
program T;
type Box<T> = record Value: T; end;
begin end.",
    );
}

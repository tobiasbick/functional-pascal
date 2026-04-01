use super::*;

// Generic type alias syntax (`Type of T`) produces a parse error.

#[test]
fn generic_type_alias_of_syntax_produces_parse_error() {
    parse_fails(
        "\
program T;
type Foo = integer;
var X: Foo of integer := 0;
begin end.",
    );
}

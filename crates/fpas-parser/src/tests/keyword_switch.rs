//! Canonical binding keywords and contextual rejection of retired modifiers.

use super::*;

#[test]
fn mutable_is_an_ordinary_case_insensitive_identifier() {
    parse_ok(
        "program T;
        function Identity(mutable: integer): integer;
        begin return MUTABLE; end function;
        var Mutable: integer := Identity(1);
        begin mutable := Mutable + 1; end.",
    );
    parse_ok(
        "program T; procedure mutable(); begin end procedure;
        begin mutable(); end.",
    );
}

#[test]
fn retired_binding_prefix_has_a_var_hint() {
    for source in [
        "program T; mutable var X: integer := 0; begin end.",
        "program T; begin MuTaBlE var X: integer := 0; end.",
        "unit T; public mutable var X: integer := 0; end unit;",
    ] {
        let (_, errors) = parse_compilation_unit_with_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.as_parser_error().is_some_and(|error| {
                    error.message.contains("binding prefix has been removed")
                        && error
                            .help
                            .as_deref()
                            .is_some_and(|hint| hint.contains("var X:"))
                })
            }),
            "{errors:#?}"
        );
    }
}

#[test]
fn retired_parameter_modifier_has_a_local_copy_hint() {
    for source in [
        "program T; procedure P(mutable X: integer); begin end procedure; begin end.",
        "program T; type F = function(mutable X: integer): integer; begin end.",
        "program T; begin const F: procedure(X: integer) := procedure(mutable X: integer) begin end procedure; end.",
        "program T; type R = record procedure P(mutable Self: R); begin end procedure; end record; begin end.",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(
            errors.iter().any(|error| {
                error.as_parser_error().is_some_and(|error| {
                    error
                        .message
                        .contains("parameter modifier has been removed")
                        && error
                            .help
                            .as_deref()
                            .is_some_and(|hint| hint.contains("var LocalValue:"))
                })
            }),
            "{errors:#?}"
        );
    }
}

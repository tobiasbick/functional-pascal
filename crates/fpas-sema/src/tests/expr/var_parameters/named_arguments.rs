//! Named `var` arguments reuse the positional storage, aliasing, and lifetime rules.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::{check_errors, check_ok, error_codes, program};
use fpas_diagnostics::codes::{
    SEMA_INVALID_NAMED_ARGUMENT, SEMA_INVALID_VAR_ARGUMENT, SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED,
    SEMA_TYPE_MISMATCH, SEMA_VAR_ARGUMENT_ALIAS, SEMA_VAR_ARGUMENT_MARKER,
    SEMA_VAR_PARAMETER_ESCAPE,
};

#[test]
fn named_var_arguments_accept_storage_in_any_parameter_order() {
    check_ok(&program(
        r#"
  var Counter: integer := 0;
  var P: Point := Point( X := 1, Y := 2 );
  var Items: array of integer := [1, 2];
  Increase(vAlUe := var Counter);
  Increase(Value := var P.X);
  Increase(Value := var Items[0]);
  Increase(Value := var Global);
  Swap(B := var P.Y, A := var Counter);
  const N: integer := Next(Counter := var Items[1]);
"#,
    ));
    check_ok(
        "program T;
procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
procedure Forward(var Value: integer); begin Increase(Value := var Value); end procedure;
begin var Counter: integer := 0; Forward(Value := var Counter); end.",
    );
}

#[test]
fn named_marker_diagnostics_show_the_corrected_argument() {
    for (call, hint) in [
        ("Increase(Value := Counter)", "Value := var Counter"),
        ("Increase(Value := 42)", "Value := var Temp"),
        ("Show(Value := var Counter)", "Value := Counter"),
    ] {
        let errors = check_errors(&program(&format!("var Counter: integer := 0; {call};")));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_VAR_ARGUMENT_MARKER);
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(hint)),
            "{errors:#?}"
        );
    }
}

#[test]
fn named_var_arguments_reject_shared_roots_in_written_order() {
    for (body, first, second) in [
        (
            "var Items: array of integer := [1, 2]; Swap(B := var Items[1], A := var Items[0]);",
            "Items[1]",
            "Items[0]",
        ),
        (
            "var P: Point := Point( X := 1, Y := 2 ); Swap(B := var P.Y, A := var P.X);",
            "P.Y",
            "P.X",
        ),
        (
            "var Counter: integer := 0; Swap(B := var Counter, A := var Counter);",
            "Counter",
            "Counter",
        ),
    ] {
        let errors = check_errors(&program(body));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_VAR_ARGUMENT_ALIAS);
        assert!(
            errors[0]
                .message
                .contains(&format!("`{first}` and `{second}`")),
            "{errors:#?}"
        );
    }
}

#[test]
fn named_var_arguments_require_writable_storage_and_exact_types() {
    for body in [
        "const Fixed: integer := 1; Increase(Value := var Fixed);",
        "var D: dict of string to integer := ['a': 1]; Increase(Value := var D['a']);",
        "for I: integer := 1 to 2 do Increase(Value := var I); end for;",
    ] {
        assert_eq!(error_codes(body), [SEMA_INVALID_VAR_ARGUMENT], "{body}");
    }
    assert_eq!(
        error_codes("var R: real := 1.0; Increase(Value := var R);"),
        [SEMA_TYPE_MISMATCH]
    );
    let errors = check_errors(
        "program T;
procedure Increase(var Value: integer); begin end procedure;
procedure Copy(Value: integer); begin Increase(Value := var Value); end procedure;
begin end.",
    );
    assert_eq!(errors[0].code, SEMA_INVALID_VAR_ARGUMENT);
}

#[test]
fn named_var_calls_keep_name_mapping_and_callable_restrictions() {
    for body in [
        "var Counter: integer := 0; Increase(Unknown := var Counter);",
        "var Counter: integer := 0; Increase(Value := var Counter, VALUE := var Counter);",
        "var Counter: integer := 0; Swap(B := var Counter);",
    ] {
        assert!(
            error_codes(body).contains(&SEMA_INVALID_NAMED_ARGUMENT),
            "{body}"
        );
    }
    assert!(error_codes("var Counter: integer := 0; const F: procedure(var Value: integer) := Increase; F(Value := var Counter);").contains(&SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED));
    assert_eq!(
        error_codes("var Counter: integer := 0; go Increase(Value := var Counter);"),
        [SEMA_VAR_PARAMETER_ESCAPE]
    );
}

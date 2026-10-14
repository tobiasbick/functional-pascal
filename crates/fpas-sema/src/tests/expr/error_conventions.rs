//! Canonical parser and error-handling contracts at the public checker boundary.
//! See `docs/pascal/std/text/parse.md` and `docs/pascal/language/error-handling/try.md`.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

#[test]
fn canonical_parsers_have_result_signatures_and_text_parameter_labels() {
    for (parser, payload, text) in [
        ("Int", "integer", "42"),
        ("Real", "real", "1.0"),
        ("Bool", "boolean", "true"),
        ("HexInt", "integer", "FF"),
    ] {
        check_ok(&format!(
            "program T; uses Std.Parse as Numbers; begin
            const Direct: Result of ({payload}, string) := Numbers.{parser}('{text}');
            const Named: Result of ({payload}, string) := Numbers.{parser}(tExT := '{text}');
            const Reader: function(Text: string): Result of ({payload}, string) := Numbers.{parser};
            const ViaValue := Reader(Text := '{text}'); end."
        ));
        let errors = check_errors(&format!(
            "program T; uses Std.Parse as Numbers; begin const Plain: {payload} := Numbers.{parser}('{text}'); end."
        ));
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
        assert!(errors[0].message.contains("Result"), "{errors:?}");
    }
}

#[test]
fn retired_parsers_and_native_helpers_explain_the_selected_replacement() {
    for (call, hint) in [
        ("Numbers.TryInt('1')", "Numbers.Int(Text)"),
        ("Numbers.TryReal('1.0')", "Numbers.Real(Text)"),
        ("Numbers.TryBool('true')", "Numbers.Bool(Text)"),
        ("Conversions.StrToInt('1')", "Numbers.Int(Text).Unwrap()"),
        (
            "Conversions.StrToReal('1.0')",
            "Numbers.Real(Text).Unwrap()",
        ),
        (
            "Conversions.StrToBool('true')",
            "Numbers.Bool(Text).Unwrap()",
        ),
        (
            "Conversions.HexToInt('FF')",
            "Numbers.HexInt(Text).Unwrap()",
        ),
        ("Some(1).IsSome()", "Value is Some(_)"),
        ("Some(1).IsNone()", "Value is None"),
        ("Ok(1).IsOk()", "Value is Ok(_)"),
        ("Ok(1).IsError()", "Value is Error(_)"),
        ("Some(1).UnwrapOr(0)", "case Value"),
        ("Ok(1).UnwrapOr(Default := 0)", "case Value"),
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Parse as Numbers, Std.Conv as Conversions; begin discard {call}; end."
        ));
        assert_eq!(errors.len(), 1, "{call}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME, "{errors:?}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(hint)),
            "{errors:?}"
        );
    }
    let errors = check_errors(
        "program T; uses Std.Conv as Convert; begin const Parse: integer := 0; discard Convert.StrToInt('1'); end.",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("uses Std.Parse as Parse2")),
        "{errors:?}"
    );
    check_ok(
        "program T; type Custom = record Value: integer; function IsSome(Self: Custom): boolean; begin return Self.Value > 0; end function; function UnwrapOr(Self: Custom; Default: integer): integer; begin return Self.Value + Default; end function; end record; function StrToInt(Text: string): integer; begin return Text.Length(); end function; begin const Value := Custom(Value := 1); discard Value.IsSome(); discard Value.UnwrapOr(2); discard StrToInt('abc'); end.",
    );
}

#[test]
fn try_matrix_preserves_payload_independence_and_error_compatibility() {
    for (input, output, value) in [
        (
            "Result of (integer, string)",
            "Result of (boolean, string)",
            "Ok(true)",
        ),
        ("Option of integer", "Option of boolean", "Some(true)"),
        (
            "Result of (integer, Failure)",
            "Result of (boolean, FailureAlias)",
            "Ok(true)",
        ),
    ] {
        check_ok(&format!(
            "program T; type Failure = distinct string; type FailureAlias = Failure; function Forward(Input: {input}): {output}; begin const Payload: integer := try Input; return {value}; end function; begin end."
        ));
    }
    check_ok(
        "program T; function Forward<T, U, E>(Input: Result of (T, E); Value: U): Result of (U, E); begin discard try Input; return Ok(Value); end function; begin end.",
    );
    for declaration in [
        "function Wrong<E>(Input: Result of (integer, E)): Result of (integer, string);",
        "function Wrong<E>(Input: Result of (integer, string)): Result of (integer, E);",
        "function Wrong<E, F>(Input: Result of (integer, E)): Result of (integer, F);",
        "function Wrong<E>(Input: Result of (integer, array of E)): Result of (integer, array of string);",
    ] {
        let errors = check_errors(&format!(
            "program T; {declaration} begin discard try Input; return Ok(1); end function; begin end."
        ));
        assert_eq!(errors.len(), 1, "{declaration}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
        assert!(errors[0].message.contains("try"), "{errors:?}");
    }
    for (input, output, value) in [
        (
            "Result of (integer, string)",
            "Option of integer",
            "Some(1)",
        ),
        ("Option of integer", "Result of (integer, string)", "Ok(1)"),
        ("Result of (integer, string)", "integer", "1"),
        ("Option of integer", "integer", "1"),
        (
            "Result of (integer, string)",
            "Result of (integer, integer)",
            "Ok(1)",
        ),
        (
            "Result of (integer, Failure)",
            "Result of (integer, string)",
            "Ok(1)",
        ),
    ] {
        let errors = check_errors(&format!(
            "program T; type Failure = distinct string; function Forward(Input: {input}): {output}; begin discard try Input; return {value}; end function; begin end."
        ));
        assert_eq!(errors.len(), 1, "{input} -> {output}: {errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
        assert!(errors[0].message.contains("try"), "{errors:?}");
    }
    for source in [
        "program T; begin discard try Some(1); end.",
        "program T; procedure P(); begin discard try Some(1); end procedure; begin end.",
        "program T; function F(): Option of integer; begin discard try 1; return None; end function; begin end.",
    ] {
        let errors = check_errors(source);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:?}");
        assert!(errors[0].message.contains("try"), "{errors:?}");
    }
}

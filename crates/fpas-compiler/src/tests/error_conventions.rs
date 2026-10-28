//! Runtime failure boundaries retained by canonical Result/Option handling.
//! See `docs/pascal/language/error-handling/try.md`.

use super::run_program;
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_UNWRAP_FAILURE};

#[test]
fn every_result_parser_returns_errors_and_explicit_unwrap_aborts() {
    for (parser, text) in [
        ("Int", "bad-int"),
        ("Real", "bad-real"),
        ("Bool", "bad-bool"),
        ("HexInt", "bad-hex"),
    ] {
        let error = run_program(&format!(
            "program T; uses Std.Text.Parse as Parse; begin discard Parse.{parser}('{text}').Unwrap(); end."
        )).expect_err("explicit parser Unwrap aborts");
        assert_eq!(error.code, RUNTIME_UNWRAP_FAILURE, "{error:?}");
        assert!(error.message.contains(text), "{error:?}");
        assert!(
            error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("case Value")),
            "{error:?}"
        );
    }
}

#[test]
fn try_does_not_catch_panic_or_unwrap_runtime_failures() {
    for (declaration, operation, code, message) in [
        (
            "function Fail(): Result of (integer, string); begin panic('inner panic'); end function;",
            "Fail()",
            RUNTIME_PROGRAM_PANIC,
            "inner panic",
        ),
        (
            "function Fail(): Option of integer; begin panic('inner panic'); end function;",
            "Fail()",
            RUNTIME_PROGRAM_PANIC,
            "inner panic",
        ),
        (
            "function Fail(): Result of (integer, string); begin return Ok(Parse.Int('bad-int').Unwrap()); end function;",
            "Fail()",
            RUNTIME_UNWRAP_FAILURE,
            "bad-int",
        ),
    ] {
        let result = if declaration.contains("Option of integer") {
            "Option of integer"
        } else {
            "Result of (integer, string)"
        };
        let error = run_program(&format!(
            "program T; uses Std.Text.Parse as Parse; {declaration} function Forward(): {result}; begin discard try {operation}; panic('unexpected continuation'); end function; begin discard Forward(); end."
        )).expect_err("runtime failures bypass try propagation");
        assert_eq!(error.code, code, "{error:?}");
        assert!(error.message.contains(message), "{error:?}");
    }
}

#[test]
fn eager_default_work_can_fail_even_with_constant_successful_payloads() {
    for (container, constructor, arms) in [
        (
            "Option of integer",
            "Some",
            "when Some(const Value): Value; when None: Default;",
        ),
        (
            "Result of (integer, string)",
            "Ok",
            "when Ok(const Value): Value; when Error(_): Default;",
        ),
    ] {
        let error = run_program(&format!(
            "program T; function DefaultValue(): integer; begin panic('eager default'); end function; begin const Saved: {container} := {constructor}(1); const Default := DefaultValue(); const Value := case Saved of {arms} end case; end."
        )).expect_err("unused successful default still has eager effects");
        assert_eq!(error.code, RUNTIME_PROGRAM_PANIC, "{error:?}");
        assert!(error.message.contains("eager default"), "{error:?}");
    }
}

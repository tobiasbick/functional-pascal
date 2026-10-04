//! Nested generic parameter declarations stay distinct during typing and inference.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn generic_parameter_identity_rejects_cross_scope_values() {
    for body in [
        "return Value;",
        "const Copy: T := Value; return Other;",
        "var Copy: T := Other; Copy := Value; return Copy;",
        "const Callback: function(): T := function(): T begin return Value; end function; return Other;",
    ] {
        let errors = check_errors(&format!(
            "program Main;
            function Outer of (T)(Value: T): integer;
              function Inner of (T)(Other: T): T; begin {body} end function;
            begin return Inner(1); end function;
            begin discard Outer('text'); end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:#?}"
        );
    }
}

#[test]
fn generic_parameter_identity_does_not_infer_enclosing_parameters() {
    for argument in ["'wrong'", "1"] {
        let errors = check_errors(&format!(
            "program Main;
            function Outer of (T)(Value: T): T;
              function Inner of (U)(Other: U; Captured: T): T;
              begin return Captured; end function;
            begin discard Inner(1, {argument}); return Value; end function;
            begin discard Outer(true); end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:#?}"
        );
    }
}

#[test]
fn generic_parameter_identity_allows_shadowing_and_preserves_outer_references() {
    check_ok(
        "program Main;
        type Box of (T) = record Value: T; end record;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        function Outer of (T)(Value: T): T;
          function Preserve(Captured: T): T; begin return Captured; end function;
          function Inner of (t)(Other: t): t;
          begin const Copy := Value; discard Preserve(Copy); return Other; end function;
        begin
          const Boxed: Box of (integer) := Box(Value := 1);
          const Selected: Choice of (integer) := Choice.Present(Boxed.Value);
          discard Selected;
          discard Inner(1);
          return Preserve(Value);
        end function;
        begin const Text: string := Outer('text'); const Number: integer := Outer(42); end program;",
    );
}

#[test]
fn generic_parameter_identity_preserves_constrained_forwarding() {
    check_ok(
        "program Main;
        function Twice of (T: Numeric)(Value: T): T; begin return Value + Value; end function;
        function Outer of (T: Numeric)(Value: T): T;
          function Inner of (T: Numeric)(Other: T): T;
          begin discard Twice(Value); return Twice(Other); end function;
        begin discard Inner(1.5); return Twice(Value); end function;
        begin const Answer: integer := Outer(21); end program;",
    );
}

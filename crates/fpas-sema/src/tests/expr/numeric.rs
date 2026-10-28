//! Numeric compatibility is uniform across operators and ordinary value boundaries.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn numeric_mixtures_require_explicit_conversion_in_every_context() {
    for body in [
        "discard 1 + 0.5;",
        "discard 1 - 0.5;",
        "discard 1 * 0.5;",
        "discard 1 / 0.5;",
        "discard 1 = 1.0;",
        "discard 1 <> 1.0;",
        "discard 1 < 1.5;",
        "discard 1 <= 1.5;",
        "discard 1 > 1.5;",
        "discard 1 >= 1.5;",
        "const R: real := 1;",
        "var R: real := 1.0; R := 1;",
        "UseReal(1);",
        "const P: Point := Point(X := 1);",
        "discard Add(1, 1.0);",
    ] {
        let source = format!(
            "program T; type Point = record X: real; end record; procedure UseReal(Value: real); begin null; end procedure; function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function; begin {body} end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{body}: {errors:?}"
        );
    }
    for (parameters, expression) in [
        ("A: T; B: real", "A + B"),
        ("A: T; B: U", "A / B"),
        ("A: T; B: integer", "A < B"),
    ] {
        let errors = check_errors(&format!(
            "program T; function F<T: Numeric, U: Numeric>({parameters}): boolean; begin discard {expression}; return true; end function; begin end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:?}"
        );
    }
    check_ok(
        "program T; uses Std.Math as Math; begin const R: real := Math.IntToReal(1) + 0.5; discard Math.IntToReal(1) < 1.5; const Half: real := 1 / 2; end.",
    );
}

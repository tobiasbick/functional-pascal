//! Numeric language contracts through compilation, optimization, and VM execution.

use super::{assert_succeeds, run_program};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;

#[test]
fn integer_overflows_and_nonfinite_real_results_fail_on_the_executed_path() {
    for expression in [
        "9223372036854775807 + 1",
        "(-9223372036854775807 - 1) - 1",
        "9223372036854775807 * 2",
        "0 - (-9223372036854775807 - 1)",
        "-(-9223372036854775807 - 1)",
        "(-9223372036854775807 - 1) div (-1)",
        "1.0e308 * 10.0",
        "1.0e308 + 1.0e308",
        "1.0e308 / 0.1",
        "Math.Exp(1000.0)",
        "Math.Pow(-1.0, 0.5)",
    ] {
        let source = format!("program T; uses Std.Math as Math; begin discard {expression}; end.");
        let error = run_program(&source).expect_err("numeric domain failure");
        assert_eq!(
            error.code, RUNTIME_NUMERIC_DOMAIN_ERROR,
            "{expression}: {error:?}"
        );
    }
    for operand in ["const", "var"] {
        let source = format!(
            "program T; begin {operand} A: integer := 9223372036854775807; {operand} B: integer := 1; discard A + B; end."
        );
        assert_eq!(
            run_program(&source).expect_err("overflow").code,
            RUNTIME_NUMERIC_DOMAIN_ERROR
        );
    }
    for arguments in ["9223372036854775807, 1", "1.0e308, 1.0e308"] {
        let generic = format!(
            "program T; function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function; begin discard Add({arguments}); end."
        );
        assert_eq!(
            run_program(&generic).expect_err("generic overflow").code,
            RUNTIME_NUMERIC_DOMAIN_ERROR
        );
    }
    assert_succeeds(
        "program T; begin if false then discard 9223372036854775807 + 1; end if; const Safe: boolean := false and ((9223372036854775807 + 1) = 0); end.",
    );
}

#[test]
fn rounding_and_numeric_text_use_the_selected_contract() {
    assert_succeeds(
        "program T; uses Std.Math as Math, Std.Conv as Conv, Std.Parse as Parse; begin\n        if Math.Round(2.5) <> 2 or Math.Round(3.5) <> 4 or Math.Round(-2.5) <> -2 or Math.Trunc(-2.9) <> -2 then panic('rounding'); end if;\n        if Conv.RealToStr(1.0) <> '1.0' or Conv.RealToStr(-0.0) <> '-0.0' then panic('real text'); end if;\n        if not Parse.TryReal(Conv.RealToStr(1.0)).IsOk() then panic('roundtrip'); end if;\n        if Parse.TryInt('$FF').Unwrap() <> 255 or Parse.TryInt('-$8000000000000000').Unwrap() <> (-9223372036854775807 - 1) then panic('hex'); end if;\n        if not Parse.TryReal('1.0e-9999').IsError() or not '$FF'.IsNumeric() then panic('numeric range'); end if;\n        const Tiny: real := 5.0e-324;\n        if Conv.RealToStr(Tiny * 0.5) <> '0.0' or Conv.RealToStr(-Tiny * 0.5) <> '-0.0' then panic('arithmetic underflow'); end if;\n    end.",
    );
}

#[test]
fn signed_minimum_remainder_and_generic_real_division_execute() {
    assert_succeeds("program T; function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function; function Divide<T: Numeric>(A: T; B: T): real; begin return A / B; end function; begin
        const Minimum: integer := -9223372036854775808;
        const HexMinimum: integer := -$8000000000000000;
        if Minimum <> HexMinimum or (Minimum mod (-1)) <> 0 or (-7 div 3) <> -2 or (-7 mod 3) <> -1 or (7 mod (-3)) <> 1 then panic('integer boundaries'); end if;
        if Divide(1, 2) <> 0.5 or Divide(1.0, 2.0) <> 0.5 then panic('generic real division'); end if;
        if Add(1, 2) <> 3 or Add(1.5, 2.5) <> 4.0 then panic('generic arithmetic'); end if;
    end.");
}

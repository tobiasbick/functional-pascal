use super::super::{assert_succeeds, parse_ok, run_program};

#[test]
fn short_circuit_and_xor_preserve_left_to_right_effects() {
    assert_succeeds(
        r#"program Order;
uses Std.Test as Test;
mutable var Trace: integer := 0;
function Mark(Digit: integer; Value: boolean): boolean;
begin
  Trace := Trace * 10 + Digit;
  return Value;
end function;
begin
  var A: boolean := Mark(1, false) and Mark(2, true);
  Test.AssertEquals(1, Trace);
  Trace := 0;
  var B: boolean := Mark(1, true) or Mark(2, false);
  Test.AssertEquals(1, Trace);
  Trace := 0;
  var C: boolean := Mark(1, false) xor Mark(2, true) xor Mark(3, true);
  Test.AssertEquals(123, Trace);
  Test.AssertEquals(false, C);
  Trace := 0;
  var D: boolean := Mark(1, true) and (Mark(2, false) or Mark(3, true));
  Test.AssertEquals(123, Trace);
  Test.AssertEquals(true, D);
  Trace := 0;
  var E: boolean := (Mark(1, false) and Mark(2, true)) or Mark(3, true);
  Test.AssertEquals(13, Trace);
  Test.AssertEquals(true, E);
end program;"#,
    );
}

#[test]
fn skipped_failures_and_constant_initializers_do_not_execute() {
    assert_succeeds(
        r#"program Skip;
uses Std.Test as Test;
const A: boolean := false and (1 div 0 = 0);
const B: boolean := true or (1 div 0 = 0);
function Crash(): boolean;
begin panic('must be skipped'); return false; end function;
begin
  Test.AssertEquals(false, A);
  Test.AssertEquals(true, B);
  Test.AssertEquals(false, false and Crash());
  Test.AssertEquals(true, true or Crash());
  Test.AssertEquals(true, not 3 < 2 and 2 + 3 * 4 = 14);
  Test.AssertEquals(false, not 1 in [1, 2]);
end program;"#,
    );
    for expression in [
        "true and (1 div 0 = 0)",
        "false or (1 div 0 = 0)",
        "false xor (1 div 0 = 0)",
    ] {
        let error = run_program(&format!(
            "program T; begin var X: boolean := {expression}; end program;"
        ))
        .unwrap_err();
        assert!(error.message.contains("zero"), "{error:?}");
    }
}

#[test]
fn rhs_try_returns_through_the_correct_continuation() {
    assert_succeeds(
        r#"program TryBoolean;
uses Std.Test as Test;
uses Std.Results as Results;
function Bad(): Result of boolean, string;
begin return Error('failure'); end function;
function Check(First: boolean): Result of boolean, string;
begin return Ok(First and try Bad()); end function;
begin
  Test.AssertEquals(false, Results.Unwrap(Check(false)));
  Test.AssertEquals(true, Results.IsError(Check(true)));
end program;"#,
    );
}

#[test]
fn bit_call_arguments_execute_once_in_written_order() {
    assert_succeeds(
        r#"program BitOrder;
uses Std.Bits as Bits;
uses Std.Test as Test;
mutable var Trace: integer := 0;
function Next(Digit: integer): integer;
begin Trace := Trace * 10 + Digit; return Digit; end function;
begin
  Test.AssertEquals(3, Bits.BitOr(Next(1), Next(2)));
  Test.AssertEquals(12, Trace);
  Trace := 0;
  Test.AssertEquals(4, Bits.ShiftLeft(Next(1), Next(2)));
  Test.AssertEquals(12, Trace);
end program;"#,
    );
}

#[test]
fn shift_count_runtime_failures_retain_call_locations() {
    for name in ["ShiftLeft", "ShiftRight"] {
        for count in [-1, 64, 65] {
            let error = run_program(&format!("program T; uses Std.Bits as Bits; begin var X: integer := Bits.{name}(0, {count}); end program;")).unwrap_err();
            assert!(error.message.contains("0..63"), "{error:?}");
        }
    }
}

#[test]
fn short_circuit_ir_is_valid_before_and_after_optimization() {
    let program = parse_ok(
        "program T; mutable var X: boolean := true; begin X := X and (false or X); end program;",
    );
    let mut ir = crate::lower(&program).unwrap();
    ir.validate().unwrap();
    crate::optimize::optimize(&mut ir);
    ir.validate().unwrap();
}

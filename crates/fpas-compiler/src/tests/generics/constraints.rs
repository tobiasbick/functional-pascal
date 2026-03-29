/// Tests for generic type parameter constraints: `<T: Comparable>`, etc.
///
/// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE — constrained generics that should compile and run
// ═══════════════════════════════════════════════════════════════

#[test]
fn constrained_record_comparable_with_integer() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type Ordered<T: Comparable> = record Value: T; end;
begin
  var O: Ordered of integer := record Value := 42; end;
  WriteLn(IntToStr(O.Value))
end.",
    );
    assert_eq!(output.lines, ["42"]);
}

#[test]
fn constrained_record_comparable_with_string() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
type Ordered<T: Comparable> = record Value: T; end;
begin
  var O: Ordered of string := record Value := 'hello'; end;
  WriteLn(O.Value)
end.",
    );
    assert_eq!(output.lines, ["hello"]);
}

#[test]
fn constrained_record_numeric_with_integer() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type NumBox<T: Numeric> = record Value: T; end;
begin
  var N: NumBox of integer := record Value := 7; end;
  WriteLn(IntToStr(N.Value))
end.",
    );
    assert_eq!(output.lines, ["7"]);
}

#[test]
fn constrained_record_numeric_with_real() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type NumBox<T: Numeric> = record Value: T; end;
begin
  var N: NumBox of real := record Value := 3.14; end;
  WriteLn(RealToStr(N.Value))
end.",
    );
    assert_eq!(output.lines, ["3.14"]);
}

#[test]
fn constrained_enum_comparable() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type Maybe<T: Comparable> = enum Just(Value: T); Nothing; end;
begin
  var M: Maybe of integer := Maybe.Just(99);
  case M of
    Maybe.Just(V): WriteLn(IntToStr(V));
    Maybe.Nothing: WriteLn('none')
  end
end.",
    );
    assert_eq!(output.lines, ["99"]);
}

#[test]
fn constrained_printable_with_record() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type Wrapper<T: Printable> = record Inner: T; end;
begin
  var W: Wrapper of integer := record Inner := 5; end;
  WriteLn(IntToStr(W.Inner))
end.",
    );
    assert_eq!(output.lines, ["5"]);
}

#[test]
fn mixed_constrained_and_unconstrained_params() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
type Pair<K: Comparable, V> = record Key: K; Value: V; end;
begin
  var P: Pair of string, integer := record Key := 'x'; Value := 10; end;
  WriteLn(P.Key + '=' + IntToStr(P.Value))
end.",
    );
    assert_eq!(output.lines, ["x=10"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — constraint violations
// ═══════════════════════════════════════════════════════════════

#[test]
fn constraint_violation_numeric_with_string() {
    let err = compile_err(
        "\
program T;
type NumBox<T: Numeric> = record Value: T; end;
begin
  var N: NumBox of string := record Value := 'oops'; end
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Numeric"),
        "expected constraint violation error, got: {}",
        err.message
    );
}

#[test]
fn constraint_violation_numeric_with_boolean() {
    let err = compile_err(
        "\
program T;
type NumBox<T: Numeric> = record Value: T; end;
begin
  var N: NumBox of boolean := record Value := true; end
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Numeric"),
        "expected constraint violation error, got: {}",
        err.message
    );
}

#[test]
fn constraint_violation_comparable_with_array() {
    let err = compile_err(
        "\
program T;
type Sorted<T: Comparable> = record Value: T; end;
begin
  var S: Sorted of array of integer := record Value := [1, 2]; end
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Comparable"),
        "expected constraint violation error, got: {}",
        err.message
    );
}

#[test]
fn unknown_constraint_name() {
    let err = compile_err(
        "\
program T;
type Box<T: Sortable> = record Value: T; end;
begin
  var B: Box of integer := record Value := 1; end
end.",
    );
    assert!(
        err.message.contains("Unknown type constraint"),
        "expected unknown constraint error, got: {}",
        err.message
    );
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — Comparable constraint with all valid types
// ═══════════════════════════════════════════════════════════════

#[test]
fn constrained_comparable_with_boolean() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
type Ordered<T: Comparable> = record Value: T; end;
begin
  var O: Ordered of boolean := record Value := true; end;
  WriteLn(O.Value)
end.",
    );
    assert_eq!(output.lines, ["true"]);
}

#[test]
fn constrained_comparable_with_char() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
type Ordered<T: Comparable> = record Value: T; end;
begin
  var O: Ordered of char := record Value := 'A'; end;
  WriteLn(O.Value)
end.",
    );
    assert_eq!(output.lines, ["A"]);
}

#[test]
fn constrained_comparable_with_real() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
type Ordered<T: Comparable> = record Value: T; end;
begin
  var O: Ordered of real := record Value := 2.5; end;
  WriteLn(O.Value)
end.",
    );
    assert_eq!(output.lines, ["2.5"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — Printable constraint violation with function type
// ═══════════════════════════════════════════════════════════════

#[test]
fn constraint_violation_printable_with_function_type() {
    let err = compile_err(
        "\
program T;
type Show<T: Printable> = record Value: T; end;
begin
  var S: Show of function(X: integer): integer := record Value := Add; end
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Printable"),
        "expected Printable constraint violation, got: {}",
        err.message
    );
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — Numeric constraint with non-numeric types
// ═══════════════════════════════════════════════════════════════

#[test]
fn constraint_violation_numeric_with_char() {
    let err = compile_err(
        "\
program T;
type NumBox<T: Numeric> = record Value: T; end;
begin
  var N: NumBox of char := record Value := 'A'; end
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Numeric"),
        "expected Numeric constraint violation, got: {}",
        err.message
    );
}

// ═══════════════════════════════════════════════════════════════
// POSITIVE — Constrained generic functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn constrained_generic_function_comparable() {
    // Comparable constraint on a function: `=` works on generic T because the VM
    // supports equality on all value types.
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
function IsEqual<T: Comparable>(A: T; B: T): boolean;
begin
  return A = B
end;
begin
  WriteLn(IsEqual(42, 42));
  WriteLn(IsEqual('a', 'b'))
end.",
    );
    assert_eq!(output.lines, ["true", "false"]);
}

#[test]
fn constrained_generic_function_numeric_arithmetic() {
    // Numeric constraint allows arithmetic operators inside the function body.
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
function Add<T: Numeric>(A: T; B: T): T;
begin
  return A + B
end;
begin
  WriteLn(IntToStr(Add(3, 4)));
  WriteLn(RealToStr(Add(1.5, 2.5)))
end.",
    );
    assert_eq!(output.lines, ["7", "4"]);
}

#[test]
fn constrained_generic_function_numeric_negate() {
    let output = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
function Neg<T: Numeric>(X: T): T;
begin
  return -X
end;
begin
  WriteLn(IntToStr(Neg(5)));
  WriteLn(RealToStr(Neg(3.14)))
end.",
    );
    assert_eq!(output.lines, ["-5", "-3.14"]);
}

#[test]
fn constrained_generic_function_comparable_lt() {
    // Comparable constraint allows `<` comparison in function body.
    let output = compile_and_run(
        "\
program T;
uses Std.Console;
function IsLess<T: Comparable>(A: T; B: T): boolean;
begin
  return A < B
end;
begin
  WriteLn(IsLess(1, 2));
  WriteLn(IsLess(10, 5))
end.",
    );
    assert_eq!(output.lines, ["true", "false"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE — Constraint violations at call sites
// ═══════════════════════════════════════════════════════════════

#[test]
fn constrained_generic_function_violation_at_callsite() {
    // Calling a `<T: Comparable>` function with an array (not Comparable).
    let err = compile_err(
        "\
program T;
function Compare<T: Comparable>(A: T; B: T): boolean;
begin
  return A = B
end;
begin
  Compare([1], [2])
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Comparable"),
        "expected constraint violation error, got: {}",
        err.message
    );
}

#[test]
fn constrained_generic_function_numeric_violation_at_callsite() {
    // Calling a `<T: Numeric>` function with a string (not Numeric).
    let err = compile_err(
        "\
program T;
function Add<T: Numeric>(A: T; B: T): T;
begin
  return A + B
end;
begin
  Add('hello', 'world')
end.",
    );
    assert!(
        err.message.contains("constraint") || err.message.contains("Numeric"),
        "expected Numeric constraint violation error, got: {}",
        err.message
    );
}

#[test]
fn unconstrained_generic_function_no_arithmetic() {
    // Without Numeric constraint, arithmetic should fail inside the body.
    let err = compile_err(
        "\
program T;
function Add<T>(A: T; B: T): T;
begin
  return A + B
end;
begin
  Add(1, 2)
end.",
    );
    assert!(
        err.message.contains("numeric"),
        "expected numeric operand error, got: {}",
        err.message
    );
}

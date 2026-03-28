use super::super::*;

// ===========================================================================
// Negative tests — type mismatches and structural errors in case-of patterns
// doc: docs/pascal/06-pattern-matching.md
// ===========================================================================

// ---------------------------------------------------------------------------
// Field-count validation for enum destructuring
// ---------------------------------------------------------------------------

#[test]
fn case_enum_destructure_too_many_bindings() {
    // Circle has 1 field (Radius), but pattern supplies 2 bindings
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
begin
  var S: Shape := Shape.Circle(5.0);
  case S of
    Shape.Circle(R, X): Std.Console.WriteLn('bad');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH
    );
    assert!(
        err.message.contains("1 field") && err.message.contains("2 were"),
        "expected field count mismatch, got: {}",
        err.message
    );
}

#[test]
fn case_enum_destructure_too_few_bindings() {
    // Rectangle has 2 fields (Width, Height), but pattern supplies 1
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
begin
  var S: Shape := Shape.Rectangle(5.0, 10.0);
  case S of
    Shape.Rectangle(W): Std.Console.WriteLn('bad');
    Shape.Circle(R): Std.Console.WriteLn('circle');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH
    );
    assert!(
        err.message.contains("2 fields") && err.message.contains("1 was"),
        "expected field count mismatch, got: {}",
        err.message
    );
}

#[test]
fn case_enum_destructure_zero_bindings_on_data_variant() {
    // Num has 1 field (Value), but pattern supplies 0
    let err = compile_err(
        "\
program T;
type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;
begin
  var E: Expr := Expr.Num(1);
  case E of
    Expr.Num(): Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH
    );
    assert!(
        err.message.contains("1 field") && err.message.contains("0 were"),
        "expected field count mismatch, got: {}",
        err.message
    );
}

#[test]
fn case_enum_destructure_three_bindings_on_two_fields() {
    // Add has 2 fields (Left, Right), but pattern supplies 3
    let err = compile_err(
        "\
program T;
type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;
begin
  var E: Expr := Expr.Add(Expr.Num(1), Expr.Num(2));
  case E of
    Expr.Add(A, B, C): Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH
    );
    assert!(
        err.message.contains("2 fields") && err.message.contains("3 were"),
        "expected field count mismatch, got: {}",
        err.message
    );
}

#[test]
fn case_nested_enum_destructure_wrong_field_count() {
    // Inner variant A has 1 field, but pattern supplies 2
    let err = compile_err(
        "\
program T;
type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;
begin
  var V: Outer := Outer.Wrap(Inner.A(1));
  case V of
    Outer.Wrap(Inner.A(X, Y)): Std.Console.WriteLn('bad');
    Outer.Wrap(Inner.B): Std.Console.WriteLn('b');
    Outer.Empty: Std.Console.WriteLn('empty')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH
    );
    assert!(
        err.message.contains("1 field") && err.message.contains("2 were"),
        "expected nested field count mismatch, got: {}",
        err.message
    );
}

#[test]
fn case_enum_correct_field_count_still_works() {
    // Sanity check: correct field count compiles and runs
    let out = compile_and_run(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
begin
  var S: Shape := Shape.Rectangle(3.0, 4.0);
  case S of
    Shape.Circle(R): Std.Console.WriteLn('circle');
    Shape.Rectangle(W, H): Std.Console.WriteLn('rect');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["rect"]);
}

#[test]
fn case_enum_hint_text_present() {
    // Error must include hint about correct field count
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
begin
  var S: Shape := Shape.Circle(5.0);
  case S of
    Shape.Circle(R, X): Std.Console.WriteLn('bad');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert!(
        err.help.as_deref().is_some_and(|h| h.contains("1 binding")),
        "error must include help text about correct binding count, got: {:?}",
        err.help
    );
}

// ---------------------------------------------------------------------------
// Type mismatch errors
// ---------------------------------------------------------------------------

#[test]
fn case_guard_string_type_is_error() {
    // Guard must be boolean, not string
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if 'hello': Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('else')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("boolean") || msg.contains("guard"),
        "expected guard type error, got: {}",
        err.message
    );
}

#[test]
fn case_guard_integer_type_is_error() {
    // Guard must be boolean, not integer expression
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if X + 1: Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('else')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("boolean") || msg.contains("guard"),
        "expected guard type error, got: {}",
        err.message
    );
}

#[test]
fn case_non_exhaustive_enum_missing_two() {
    // Missing Yellow AND Green
    let err = compile_err(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Red;
  case L of
    Light.Red: Std.Console.WriteLn('stop')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        err.message.contains("Yellow"),
        "should mention Yellow, got: {}",
        err.message
    );
    assert!(
        err.message.contains("Green"),
        "should mention Green, got: {}",
        err.message
    );
}

#[test]
fn case_non_exhaustive_option_missing_none() {
    let err = compile_err(
        "\
program T;
begin
  var O: Option of integer := Some(1);
  case O of
    Some(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        err.message.contains("None"),
        "should mention None, got: {}",
        err.message
    );
}

#[test]
fn case_non_exhaustive_result_missing_error() {
    let err = compile_err(
        "\
program T;
begin
  var R: Result of integer, string := Ok(1);
  case R of
    Ok(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        err.message.contains("Error"),
        "should mention Error, got: {}",
        err.message
    );
}

#[test]
fn case_non_exhaustive_guard_only_not_sufficient() {
    // A guarded arm alone does not count toward exhaustiveness
    let err = compile_err(
        "\
program T;
type Dir = enum North; South; end;
begin
  var D: Dir := Dir.North;
  case D of
    Dir.North if true: Std.Console.WriteLn('north');
    Dir.South: Std.Console.WriteLn('south')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(
        err.message.contains("North"),
        "should mention North, got: {}",
        err.message
    );
}

#[test]
fn case_type_mismatch_boolean_label_on_integer() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    true: Std.Console.WriteLn('bad')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("type") || msg.contains("mismatch") || msg.contains("compat"),
        "expected type error, got: {}",
        err.message
    );
}

#[test]
fn nested_pattern_non_exhaustive_outer_missing() {
    // Nested patterns: missing Outer.Empty variant
    let err = compile_err(
        "\
program T;
type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;
begin
  var V: Outer := Outer.Wrap(Inner.A(1));
  case V of
    Outer.Wrap(Inner.A(X)):
      Std.Console.WriteLn('a');
    Outer.Wrap(Inner.B):
      Std.Console.WriteLn('b')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("non-exhaustive") || msg.contains("missing") || msg.contains("empty"),
        "expected non-exhaustive error, got: {}",
        err.message
    );
}

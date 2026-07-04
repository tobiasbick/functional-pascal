use super::super::super::*;

// ===========================================================================
// Negative tests — type mismatches and structural errors in case-of patterns
// doc: docs/pascal/language/pattern-matching/README.md
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
    // Nested enum patterns are not supported — any nested pattern is rejected
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
  var V: Outer := Outer.Empty;
  case V of
    Outer.Wrap(Inner.A(X)): Std.Console.WriteLn('bad');
    Outer.Empty: Std.Console.WriteLn('empty')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message
            .contains("Nested enum patterns are not supported"),
        "expected nested pattern rejection, got: {}",
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

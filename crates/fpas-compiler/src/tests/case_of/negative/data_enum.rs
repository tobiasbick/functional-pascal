use super::super::super::*;

#[test]
fn case_data_enum_rejects_foreign_root_variant() {
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
  Other = enum
    Square(Size: real);
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Other.Square(Size): Std.Console.WriteLn('bad');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("does not belong") || err.message.contains("Shape"),
        "expected foreign root variant error, got: {}",
        err.message
    );
}

#[test]
fn case_data_enum_rejects_foreign_nested_variant() {
    let err = compile_err(
        "\
program T;
type
  Inner = enum
    A(X: integer);
  end;
  Other = enum
    B(X: integer);
  end;
  Outer = enum
    Wrap(Value: Inner);
    Empty;
  end;
begin
  var V: Outer := Outer.Empty;
  case V of
    Outer.Wrap(Other.B(X)): Std.Console.WriteLn('bad');
    Outer.Empty: Std.Console.WriteLn('empty')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message
            .contains("Nested enum patterns are not supported")
            || err.message.contains("single-level"),
        "expected nested pattern rejection, got: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------
// Range on data-enum is not supported
// doc: docs/pascal/language/pattern-matching/README.md
// ---------------------------------------------------------------------------

#[test]
fn case_data_enum_rejects_range_label() {
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Shape.Circle(1.0)..Shape.Circle(10.0): Std.Console.WriteLn('bad');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("range")
            || err.message.contains("Range")
            || err.message.contains("do not support"),
        "expected range-on-data-enum error, got: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------
// Destructure patterns (Ok/Error/Some/None) on non-Result/Option types
// doc: docs/pascal/language/pattern-matching/README.md
// ---------------------------------------------------------------------------

#[test]
fn case_destructure_on_integer_rejected() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    Ok(V): Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('ok')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Result")
            || err.message.contains("Option")
            || err.message.contains("Destructure"),
        "expected destructure-on-integer error, got: {}",
        err.message
    );
}

#[test]
fn case_destructure_some_on_string_rejected() {
    let err = compile_err(
        "\
program T;
begin
  var S: string := 'hello';
  case S of
    Some(V): Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('ok')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Result")
            || err.message.contains("Option")
            || err.message.contains("Destructure"),
        "expected destructure-on-string error, got: {}",
        err.message
    );
}

#[test]
fn case_result_rejects_option_pattern() {
    let err = compile_err(
        "\
program T;
begin
  var R: Result of integer, string := Ok(1);
  case R of
    Some(V): Std.Console.WriteLn('bad');
    Error(E): Std.Console.WriteLn('err')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Result")
            || err
                .help
                .as_deref()
                .is_some_and(|help| help.contains("Result")),
        "expected Result/Option variant mismatch, got: {err:?}"
    );
}

#[test]
fn case_destructure_on_simple_enum_rejected() {
    let err = compile_err(
        "\
program T;
type Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Red;
  case C of
    Ok(V): Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('ok')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("result") || msg.contains("option") || msg.contains("destructure"),
        "expected destructure-on-enum error, got: {}",
        err.message
    );
}

#[test]
fn case_data_enum_rejects_wrong_literal_type_in_pattern() {
    let err = compile_err(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Shape.Circle('large'): Std.Console.WriteLn('bad');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message
            .contains("Literal matching inside enum patterns is not supported")
            || err.message.contains("guard clause"),
        "expected literal-in-pattern rejection, got: {}",
        err.message
    );
}

#[test]
fn case_option_rejects_result_pattern() {
    let err = compile_err(
        "\
program T;
begin
  var O: Option of integer := None;
  case O of
    Ok(V): Std.Console.WriteLn('bad');
    None: Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Option")
            || err
                .help
                .as_deref()
                .is_some_and(|help| help.contains("Option")),
        "expected Result/Option mismatch, got: {err:?}"
    );
}

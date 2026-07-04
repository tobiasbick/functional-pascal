use super::super::super::*;

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
    // Non-exhaustive data-enum: missing Outer.Empty variant (no nested patterns)
    let err = compile_err(
        "\
program T;
type
  Outer = enum
    Wrap(I: integer);
    Empty;
  end;
begin
  var V: Outer := Outer.Wrap(1);
  case V of
    Outer.Wrap(X):
      Std.Console.WriteLn('wrapped')
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

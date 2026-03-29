use super::*;

// ── Negative tests (sema / compile errors) ──────────────────

#[test]
fn enum_data_wrong_arg_count_too_few() {
    let err = compile_err(
        "\
program T;
type Shape = enum Circle(Radius: real); end;
begin
  var S: Shape := Shape.Circle()
end.",
    );
    assert!(
        err.message.contains("argument") || err.message.contains("Argument"),
        "expected argument-count error, got: {}",
        err.message
    );
}

#[test]
fn enum_data_wrong_arg_count_too_many() {
    let err = compile_err(
        "\
program T;
type Shape = enum Circle(Radius: real); end;
begin
  var S: Shape := Shape.Circle(1.0, 2.0)
end.",
    );
    assert!(
        err.message.contains("argument") || err.message.contains("Argument"),
        "expected argument-count error, got: {}",
        err.message
    );
}

#[test]
fn enum_data_wrong_arg_type() {
    let err = compile_err(
        "\
program T;
type Shape = enum Circle(Radius: real); end;
begin
  var S: Shape := Shape.Circle('not a number')
end.",
    );
    assert!(
        err.message.to_lowercase().contains("type")
            || err.message.to_lowercase().contains("mismatch"),
        "expected type error, got: {}",
        err.message,
    );
}

#[test]
fn enum_data_call_fieldless_variant_with_args() {
    let err = compile_err(
        "\
program T;
type Token = enum Eof; Number(V: integer); end;
begin
  var T: Token := Token.Eof(42)
end.",
    );
    // Eof has no fields, calling it with args is an error
    assert!(
        !err.message.is_empty(),
        "expected error, got: {}",
        err.message
    );
}

#[test]
fn unknown_enum_variant() {
    let err = compile_err(
        "\
program T;
type Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Yellow
end.",
    );
    assert!(
        err.message.contains("Yellow")
            || err.message.to_lowercase().contains("unknown")
            || err.message.to_lowercase().contains("variant"),
        "expected unknown variant error, got: {}",
        err.message
    );
}

#[test]
fn enum_type_mismatch_in_assignment() {
    let err = compile_err(
        "\
program T;
type Color = enum Red; Green; Blue; end;
type Dir = enum North; South; end;
begin
  var C: Color := Dir.North
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn enum_data_variant_used_as_simple_when_it_has_fields() {
    // Currently, the compiler treats `Shape.Circle` (without args) as a reference
    // to the variant constructor rather than a compile error.  This compiles but
    // produces a function value — the variant is not "called".
    compile_ok(
        "\
program T;
type Shape = enum Circle(Radius: real); end;
begin
  var S: Shape := Shape.Circle
end.",
    );
}

#[test]
fn enum_returned_from_function_and_matched() {
    let out = compile_and_run(
        "\
program EnumReturnMatch;
uses Std.Console;
type Dir = enum North; South; East; West; end;

function Opposite(D: Dir): Dir;
begin
  case D of
    Dir.North: return Dir.South;
    Dir.South: return Dir.North;
    Dir.East: return Dir.West;
    Dir.West: return Dir.East
  end;
  return Dir.North
end;

begin
  var D: Dir := Opposite(Dir.East);
  if D = Dir.West then
    WriteLn('west')
  else
    WriteLn('other')
end.",
    );
    assert_eq!(out.lines, vec!["west"]);
}

use super::super::*;

#[test]
fn case_simple_enum() {
    let out = compile_and_run(
        "\
program T;
type
  Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Green;
  case C of
    Color.Red:   Std.Console.WriteLn('red');
    Color.Green: Std.Console.WriteLn('green');
    Color.Blue:  Std.Console.WriteLn('blue')
  end
end.",
    );
    assert_eq!(out.lines, vec!["green"]);
}

#[test]
fn case_simple_enum_with_else() {
    let out = compile_and_run(
        "\
program T;
type
  Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Blue;
  case C of
    Color.Red:   Std.Console.WriteLn('red');
    Color.Green: Std.Console.WriteLn('green')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

#[test]
fn case_enum_non_exhaustive_error() {
    let err = compile_err(
        "\
program T;
type
  Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Blue;
  case C of
    Color.Red:   Std.Console.WriteLn('red');
    Color.Green: Std.Console.WriteLn('green')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("non-exhaustive") || msg.contains("missing"),
        "expected exhaustiveness error, got: {}",
        err.message
    );
}

use super::*;

#[test]
fn enum_assign_and_compare() {
    let out = compile_and_run(
        "\
program EnumBasic;
type Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Green;
  if C = Color.Green then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}

#[test]
fn enum_case_match() {
    let out = compile_and_run(
        "\
program EnumCase;
type Dir = enum North; East; South; West; end;
begin
  var D: Dir := Dir.South;
  case D of
    Dir.North: Std.Console.WriteLn('N');
    Dir.East: Std.Console.WriteLn('E');
    Dir.South: Std.Console.WriteLn('S');
    Dir.West: Std.Console.WriteLn('W')
  end
end.",
    );
    assert_eq!(out.lines, vec!["S"]);
}

#[test]
fn enum_with_backing_values() {
    let out = compile_and_run(
        "\
program EnumBacking;
type Http = enum Success = 200; NotFound = 404; ServerError = 500; end;
begin
  var S: Http := Http.NotFound;
  if S = Http.NotFound then
    Std.Console.WriteLn('404')
  else
    Std.Console.WriteLn('other')
end.",
    );
    assert_eq!(out.lines, vec!["404"]);
}

#[test]
fn enum_pass_to_function() {
    let out = compile_and_run(
        "\
program EnumFunc;
type Color = enum Red; Green; Blue; end;

function IsRed(C: Color): boolean;
begin
  return C = Color.Red
end;

begin
  Std.Console.WriteLn(IsRed(Color.Red));
  Std.Console.WriteLn(IsRed(Color.Blue))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn enum_not_equal() {
    let out = compile_and_run(
        "\
program EnumNeq;
type Color = enum Red; Green; Blue; end;
begin
  var C: Color := Color.Red;
  if C <> Color.Blue then
    Std.Console.WriteLn('diff')
  else
    Std.Console.WriteLn('same')
end.",
    );
    assert_eq!(out.lines, vec!["diff"]);
}

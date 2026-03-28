use super::*;

#[test]
fn for_continue_with_inner_scope_vars() {
    let out = compile_and_run(
        "\
program ForContinueScope;
uses Std.Array;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5];
  for I: integer := 0 to Std.Array.Length(Arr) - 1 do
  begin
    var S: integer := Arr[I];
    if S mod 2 = 0 then continue;
    Std.Console.WriteLn(S)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn for_in_continue_with_inner_scope_vars() {
    let out = compile_and_run(
        "\
program ForInContinueScope;
begin
  var Arr: array of integer := [10, 3, 45, 7, 88];
  for S: integer in Arr do
  begin
    var Label: string := 'score';
    if S <= 10 then continue;
    Std.Console.WriteLn(Label);
    Std.Console.WriteLn(S)
  end
end.",
    );
    assert_eq!(out.lines, vec!["score", "45", "score", "88"]);
}

#[test]
fn for_break_with_inner_scope_vars() {
    let out = compile_and_run(
        "\
program ForBreakScope;
uses Std.Array;
begin
  var Arr: array of integer := [1, 2, 3, 4, 5];
  for I: integer := 0 to Std.Array.Length(Arr) - 1 do
  begin
    var S: integer := Arr[I] * 10;
    if S >= 30 then break;
    Std.Console.WriteLn(S)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}

#[test]
fn for_in_break_with_inner_scope_vars() {
    let out = compile_and_run(
        "\
program ForInBreakScope;
begin
  var Arr: array of integer := [5, 10, 99, 1];
  for X: integer in Arr do
  begin
    var Label: string := 'v';
    if X > 50 then break;
    Std.Console.WriteLn(Label);
    Std.Console.WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["v", "5", "v", "10"]);
}

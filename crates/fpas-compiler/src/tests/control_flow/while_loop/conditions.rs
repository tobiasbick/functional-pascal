use super::*;

#[test]
fn while_and_condition() {
    let out = compile_and_run(
        "\
program WhileAnd;
begin
  mutable var I: integer := 0;
  while (I >= 0) and (I < 3) do
  begin
    Std.Console.WriteLn(I);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn while_or_condition() {
    let out = compile_and_run(
        "\
program WhileOr;
begin
  mutable var A: integer := 0;
  mutable var Done: boolean := false;
  while (A < 3) or (not Done) do
  begin
    Std.Console.WriteLn(A);
    A := A + 1;
    if A >= 3 then
      Done := true
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn while_not_condition() {
    let out = compile_and_run(
        "\
program WhileNot;
begin
  mutable var Stop: boolean := false;
  mutable var I: integer := 0;
  while not Stop do
  begin
    Std.Console.WriteLn(I);
    I := I + 1;
    if I = 3 then
      Stop := true
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn while_with_if_else_body() {
    let out = compile_and_run(
        "\
program WhileIfElse;
begin
  mutable var I: integer := 1;
  while I <= 5 do
  begin
    if I mod 2 = 0 then
      Std.Console.WriteLn('even')
    else
      Std.Console.WriteLn('odd');
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["odd", "even", "odd", "even", "odd"]);
}

#[test]
fn while_string_condition() {
    let out = compile_and_run(
        "\
program WhileStr;
begin
  mutable var S: string := 'aaa';
  while S <> 'done' do
  begin
    Std.Console.WriteLn(S);
    S := 'done'
  end;
  Std.Console.WriteLn('finished')
end.",
    );
    assert_eq!(out.lines, vec!["aaa", "finished"]);
}

#[test]
fn while_condition_reevaluated() {
    let out = compile_and_run(
        "\
program WhileReeval;
begin
  mutable var Limit: integer := 3;
  mutable var I: integer := 0;
  while I < Limit do
  begin
    Std.Console.WriteLn(I);
    I := I + 1;
    if I = 2 then
      Limit := 2
  end
end.",
    );
    assert_eq!(out.lines, vec!["0", "1"]);
}

use super::*;

#[test]
fn while_false_never_executes() {
    let out = compile_and_run(
        "\
program WhileFalse;
begin
  while false do
    Std.Console.WriteLn('should not print');
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn while_single_iteration() {
    let out = compile_and_run(
        "\
program WhileOnce;
begin
  mutable var I: integer := 0;
  while I < 1 do
  begin
    Std.Console.WriteLn(I);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn while_single_statement_body() {
    let out = compile_and_run(
        "\
program WhileSingle;
begin
  mutable var I: integer := 3;
  while I > 0 do
    I := I - 1;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn while_countdown() {
    let out = compile_and_run(
        "\
program WhileDown;
begin
  mutable var I: integer := 3;
  while I > 0 do
  begin
    Std.Console.WriteLn(I);
    I := I - 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["3", "2", "1"]);
}

#[test]
fn while_true_immediate_break() {
    let out = compile_and_run(
        "\
program WhileImmediateBreak;
begin
  while true do
    break;
  Std.Console.WriteLn('after')
end.",
    );
    assert_eq!(out.lines, vec!["after"]);
}

#[test]
fn while_empty_body() {
    let out = compile_and_run(
        "\
program WhileEmptyBody;
begin
  mutable var I: integer := 3;
  while I > 0 do
  begin
    I := I - 1
  end;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn while_negative_counter() {
    let out = compile_and_run(
        "\
program WhileNeg;
begin
  mutable var I: integer := -3;
  while I < 0 do
  begin
    Std.Console.WriteLn(I);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["-3", "-2", "-1"]);
}

#[test]
fn while_many_iterations() {
    let out = compile_and_run(
        "\
program WhileMany;
begin
  mutable var I: integer := 0;
  while I < 1000 do
    I := I + 1;
  Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["1000"]);
}

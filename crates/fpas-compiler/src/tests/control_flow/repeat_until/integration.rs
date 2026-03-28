use super::*;

#[test]
fn repeat_with_function_call_in_body() {
    let out = compile_and_run(
        "\
program RepeatFn;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  mutable var I: integer := 1;
  repeat
    Std.Console.WriteLn(Double(I));
    I := I + 1
  until I > 3
end.",
    );
    assert_eq!(out.lines, vec!["2", "4", "6"]);
}

#[test]
fn repeat_with_function_call_in_condition() {
    let out = compile_and_run(
        "\
program RepeatFnCond;

function IsDone(X: integer): boolean;
begin
  return X >= 3
end;

begin
  mutable var I: integer := 0;
  repeat
    Std.Console.WriteLn(I);
    I := I + 1
  until IsDone(I)
end.",
    );
    assert_eq!(out.lines, vec!["0", "1", "2"]);
}

#[test]
fn repeat_with_case_in_body() {
    let out = compile_and_run(
        "\
program RepeatCase;
begin
  mutable var I: integer := 0;
  repeat
    case I of
      0: Std.Console.WriteLn('zero');
      1: Std.Console.WriteLn('one');
      2: Std.Console.WriteLn('two');
    end;
    I := I + 1
  until I = 3
end.",
    );
    assert_eq!(out.lines, vec!["zero", "one", "two"]);
}

#[test]
fn repeat_with_if_else_in_body() {
    let out = compile_and_run(
        "\
program RepeatIfElse;
begin
  mutable var I: integer := 0;
  repeat
    if I mod 2 = 0 then
      Std.Console.WriteLn('even')
    else
      Std.Console.WriteLn('odd');
    I := I + 1
  until I = 4
end.",
    );
    assert_eq!(out.lines, vec!["even", "odd", "even", "odd"]);
}

use super::super::*;

#[test]
fn case_integer_exact_zero() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 0;
  case X of
    0: Std.Console.WriteLn('zero');
    1: Std.Console.WriteLn('one')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["zero"]);
}

#[test]
fn case_integer_negative_value() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := -5;
  case X of
    -10: Std.Console.WriteLn('minus ten');
    -5:  Std.Console.WriteLn('minus five');
    0:   Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["minus five"]);
}

#[test]
fn case_integer_multiple_labels() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 3;
  case X of
    1, 2: Std.Console.WriteLn('low');
    3, 4: Std.Console.WriteLn('mid');
    5:    Std.Console.WriteLn('high')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["mid"]);
}

#[test]
fn case_integer_no_match_no_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 99;
  case X of
    1: Std.Console.WriteLn('one');
    2: Std.Console.WriteLn('two')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn case_only_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 42;
  case X of
    0: Std.Console.WriteLn('zero')
  else
    Std.Console.WriteLn('not zero')
  end
end.",
    );
    assert_eq!(out.lines, vec!["not zero"]);
}

#[test]
fn case_first_arm_wins() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    1..10: Std.Console.WriteLn('range');
    5:     Std.Console.WriteLn('exact')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["range"]);
}

#[test]
fn case_many_arms() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 7;
  case X of
    1: Std.Console.WriteLn('1');
    2: Std.Console.WriteLn('2');
    3: Std.Console.WriteLn('3');
    4: Std.Console.WriteLn('4');
    5: Std.Console.WriteLn('5');
    6: Std.Console.WriteLn('6');
    7: Std.Console.WriteLn('7');
    8: Std.Console.WriteLn('8');
    9: Std.Console.WriteLn('9');
    10: Std.Console.WriteLn('10')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn case_on_function_result() {
    let out = compile_and_run(
        "\
program T;

function GetValue(): integer;
begin
  return 3
end;

begin
  case GetValue() of
    1: Std.Console.WriteLn('one');
    2: Std.Console.WriteLn('two');
    3: Std.Console.WriteLn('three')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["three"]);
}

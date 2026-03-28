use super::*;

#[test]
fn case_range_middle() {
    let out = compile_and_run(
        "\
program CaseRange;
begin
  var X: integer := 5;
  case X of
    1..10: Std.Console.WriteLn('low');
    11..20: Std.Console.WriteLn('high')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["low"]);
}

#[test]
fn case_range_boundaries() {
    let out = compile_and_run(
        "\
program CaseRangeBnd;
begin
  var A: integer := 1;
  var B: integer := 10;
  var C: integer := 11;
  case A of
    1..10: Std.Console.WriteLn('A ok')
  else
    Std.Console.WriteLn('A fail')
  end;
  case B of
    1..10: Std.Console.WriteLn('B ok')
  else
    Std.Console.WriteLn('B fail')
  end;
  case C of
    1..10: Std.Console.WriteLn('C fail')
  else
    Std.Console.WriteLn('C ok')
  end
end.",
    );
    assert_eq!(out.lines, vec!["A ok", "B ok", "C ok"]);
}

#[test]
fn case_range_and_exact_mixed() {
    let out = compile_and_run(
        "\
program CaseMixed;
begin
  var X: integer := 42;
  case X of
    0: Std.Console.WriteLn('zero');
    1..10: Std.Console.WriteLn('low');
    42: Std.Console.WriteLn('answer');
    100..200: Std.Console.WriteLn('high')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["answer"]);
}

#[test]
fn case_range_else_fallthrough() {
    let out = compile_and_run(
        "\
program CaseElse;
begin
  var X: integer := 50;
  case X of
    1..10: Std.Console.WriteLn('low');
    11..20: Std.Console.WriteLn('mid')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

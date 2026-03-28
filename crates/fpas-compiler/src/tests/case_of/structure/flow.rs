use super::*;

#[test]
fn case_in_function_returns() {
    let out = compile_and_run(
        "\
program T;

function Grade(Score: integer): string;
begin
  case Score of
    0..59:   return 'F';
    60..69:  return 'D';
    70..79:  return 'C';
    80..89:  return 'B';
    90..100: return 'A'
  end;
  return '?'
end;

begin
  Std.Console.WriteLn(Grade(92));
  Std.Console.WriteLn(Grade(45));
  Std.Console.WriteLn(Grade(75));
  Std.Console.WriteLn(Grade(200))
end.",
    );
    assert_eq!(out.lines, vec!["A", "F", "C", "?"]);
}

#[test]
fn case_nested_case() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 1;
  var Y: string := 'b';
  case X of
    1:
    begin
      case Y of
        'a': Std.Console.WriteLn('1a');
        'b': Std.Console.WriteLn('1b')
      else
        Std.Console.WriteLn('1?')
      end
    end;
    2: Std.Console.WriteLn('two')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["1b"]);
}

#[test]
fn case_inside_loop() {
    let out = compile_and_run(
        "\
program T;
begin
  for I: integer := 1 to 4 do
  begin
    case I of
      1: Std.Console.WriteLn('one');
      2: Std.Console.WriteLn('two');
      3: Std.Console.WriteLn('three')
    else
      Std.Console.WriteLn('big')
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["one", "two", "three", "big"]);
}

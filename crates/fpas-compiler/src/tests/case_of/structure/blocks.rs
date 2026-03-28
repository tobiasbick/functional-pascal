use super::*;

#[test]
fn case_with_begin_end_blocks() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 2;
  case X of
    1:
    begin
      Std.Console.WriteLn('one-a');
      Std.Console.WriteLn('one-b')
    end;
    2:
    begin
      Std.Console.WriteLn('two-a');
      Std.Console.WriteLn('two-b')
    end
  else
    begin
      Std.Console.WriteLn('else-a');
      Std.Console.WriteLn('else-b')
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["two-a", "two-b"]);
}

#[test]
fn multiple_case_statements() {
    let out = compile_and_run(
        "\
program T;
begin
  var A: integer := 1;
  var B: string := 'hi';
  case A of
    1: Std.Console.WriteLn('A=1')
  else
    Std.Console.WriteLn('A=other')
  end;
  case B of
    'hi':    Std.Console.WriteLn('B=hi');
    'hello': Std.Console.WriteLn('B=hello')
  else
    Std.Console.WriteLn('B=other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["A=1", "B=hi"]);
}

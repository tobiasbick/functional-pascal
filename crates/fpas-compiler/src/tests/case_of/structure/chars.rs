use super::*;

#[test]
fn case_char_multiple_arms() {
    let out = compile_and_run(
        "\
program T;
var
  C: char := 'C';
begin
  case C of
    'A': Std.Console.WriteLn('first');
    'B': Std.Console.WriteLn('second');
    'C': Std.Console.WriteLn('third')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["third"]);
}

#[test]
fn case_char_else_fallback() {
    let out = compile_and_run(
        "\
program T;
var
  C: char := 'Z';
begin
  case C of
    'A': Std.Console.WriteLn('first')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

#[test]
fn case_char_no_match_no_else() {
    let out = compile_and_run(
        "\
program T;
var
  C: char := 'Z';
begin
  case C of
    'A': Std.Console.WriteLn('found A')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

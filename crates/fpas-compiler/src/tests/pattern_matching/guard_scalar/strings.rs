use super::*;

#[test]
fn guard_on_string_case() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'hello';
  case S of
    'hello' if false:
      Std.Console.WriteLn('guarded');
    'hello':
      Std.Console.WriteLn('plain')
  else
    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["plain"]);
}

#[test]
fn string_case_single_char_label_no_match() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'x';
  case S of
    'a': Std.Console.WriteLn('a')
  end
end.",
    );
    assert!(out.lines.is_empty());
}

#[test]
fn string_case_single_char_label_match() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'a';
  case S of
    'a': Std.Console.WriteLn('hit')
  end
end.",
    );
    assert_eq!(out.lines, vec!["hit"]);
}

#[test]
fn string_case_single_char_multiple_labels() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'b';
  case S of
    'a': Std.Console.WriteLn('a');
    'b': Std.Console.WriteLn('b');
    'c': Std.Console.WriteLn('c')
  end
end.",
    );
    assert_eq!(out.lines, vec!["b"]);
}

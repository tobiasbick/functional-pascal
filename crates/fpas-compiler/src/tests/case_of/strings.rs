use super::super::*;

#[test]
fn case_string_basic() {
    let out = compile_and_run(
        "\
program T;
begin
  var Day: string := 'Monday';
  case Day of
    'Monday':    Std.Console.WriteLn('Start of week');
    'Friday':    Std.Console.WriteLn('Almost weekend')
  else
    Std.Console.WriteLn('Midweek')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Start of week"]);
}

#[test]
fn case_string_else_branch() {
    let out = compile_and_run(
        "\
program T;
begin
  var Day: string := 'Wednesday';
  case Day of
    'Monday':    Std.Console.WriteLn('Start of week');
    'Friday':    Std.Console.WriteLn('Almost weekend')
  else
    Std.Console.WriteLn('Midweek')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Midweek"]);
}

#[test]
fn case_string_multiple_labels_per_arm() {
    let out = compile_and_run(
        "\
program T;
begin
  var Day: string := 'Sunday';
  case Day of
    'Monday':             Std.Console.WriteLn('Start');
    'Saturday', 'Sunday': Std.Console.WriteLn('Weekend')
  else
    Std.Console.WriteLn('Other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Weekend"]);
}

#[test]
fn case_string_multiple_labels_first_match() {
    let out = compile_and_run(
        "\
program T;
begin
  var Day: string := 'Saturday';
  case Day of
    'Saturday', 'Sunday': Std.Console.WriteLn('Weekend')
  else
    Std.Console.WriteLn('Other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Weekend"]);
}

#[test]
fn case_string_empty_string() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := '';
  case S of
    '': Std.Console.WriteLn('empty');
    'hello': Std.Console.WriteLn('hello')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["empty"]);
}

#[test]
fn case_string_no_match_no_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'xyz';
  case S of
    'a': Std.Console.WriteLn('found a');
    'b': Std.Console.WriteLn('found b')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

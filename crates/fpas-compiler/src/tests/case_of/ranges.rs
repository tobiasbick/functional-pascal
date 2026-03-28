use super::super::*;

#[test]
fn case_range_grade_example() {
    let out = compile_and_run(
        "\
program T;
begin
  var Score: integer := 85;
  mutable var Grade: string := '';
  case Score of
    0..59:    Grade := 'F';
    60..69:   Grade := 'D';
    70..79:   Grade := 'C';
    80..89:   Grade := 'B';
    90..100:  Grade := 'A'
  end;
  Std.Console.WriteLn(Grade)
end.",
    );
    assert_eq!(out.lines, vec!["B"]);
}

#[test]
fn case_range_lower_boundary() {
    let out = compile_and_run(
        "\
program T;
begin
  var Score: integer := 60;
  mutable var Grade: string := '';
  case Score of
    0..59:   Grade := 'F';
    60..69:  Grade := 'D';
    70..79:  Grade := 'C'
  end;
  Std.Console.WriteLn(Grade)
end.",
    );
    assert_eq!(out.lines, vec!["D"]);
}

#[test]
fn case_range_upper_boundary() {
    let out = compile_and_run(
        "\
program T;
begin
  var Score: integer := 69;
  mutable var Grade: string := '';
  case Score of
    0..59:   Grade := 'F';
    60..69:  Grade := 'D';
    70..79:  Grade := 'C'
  end;
  Std.Console.WriteLn(Grade)
end.",
    );
    assert_eq!(out.lines, vec!["D"]);
}

#[test]
fn case_range_no_match_falls_through() {
    let out = compile_and_run(
        "\
program T;
begin
  var Score: integer := 200;
  mutable var Grade: string := 'none';
  case Score of
    0..59:    Grade := 'F';
    60..100:  Grade := 'pass'
  end;
  Std.Console.WriteLn(Grade)
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

#[test]
fn case_range_mixed_with_exact_and_labels() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 0;
  case X of
    -100..-1: Std.Console.WriteLn('negative');
    0:        Std.Console.WriteLn('zero');
    1..100:   Std.Console.WriteLn('positive')
  else
    Std.Console.WriteLn('huge')
  end
end.",
    );
    assert_eq!(out.lines, vec!["zero"]);
}

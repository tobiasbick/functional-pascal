use super::super::*;

#[test]
fn short_str_ops() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Length('Hello'));
  WriteLn(ToUpper('ab'));
  WriteLn(ToLower('AB'));
  WriteLn(Trim('  x  '));
  WriteLn(Contains('abc', 'b'));
  WriteLn(StartsWith('abc', 'ab'));
  WriteLn(EndsWith('abc', 'bc'));
  WriteLn(Substring('Hello', 0, 3));
  WriteLn(IndexOf('aba', 'a'));
  WriteLn(Replace('aaa', 'a', 'b'));
  WriteLn(Join(['x', 'y'], ':'));
  WriteLn(IsNumeric('42'));
  WriteLn(IsNumeric('nope'))
end.",
    );
    assert_eq!(
        out.lines,
        vec![
            "5", "AB", "ab", "x", "true", "true", "true", "Hel", "0", "bbb", "x:y", "true",
            "false",
        ]
    );
}

#[test]
fn short_mixed_with_qualified() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Std.Math.Sqrt(Pi));
  Std.Console.WriteLn(Sqrt(16.0))
end.",
    );
    assert_eq!(out.lines.len(), 2);
    assert!(out.lines[0].starts_with("1.77"));
    assert_eq!(out.lines[1], "4");
}

#[test]
fn short_no_uses_falls_through_to_user_function() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

function Double(N: integer): integer;
begin
  return N * 2
end;

begin
  WriteLn(Double(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn short_does_not_shadow_local_variable() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Math;
begin
  var LocalPi: integer := 42;
  WriteLn(LocalPi)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

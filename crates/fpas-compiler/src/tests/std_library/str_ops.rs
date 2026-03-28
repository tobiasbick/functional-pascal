use super::*;

#[test]
fn std_str_ops() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Str.Length('Hello'));
  Std.Console.WriteLn(Std.Str.ToUpper('ab'));
  Std.Console.WriteLn(Std.Str.ToLower('AB'));
  Std.Console.WriteLn(Std.Str.Trim('  x  '));
  Std.Console.WriteLn(Std.Str.Contains('abc', 'b'));
  Std.Console.WriteLn(Std.Str.StartsWith('abc', 'ab'));
  Std.Console.WriteLn(Std.Str.EndsWith('abc', 'bc'));
  Std.Console.WriteLn(Std.Str.Substring('Hello', 0, 3));
  Std.Console.WriteLn(Std.Str.IndexOf('aba', 'a'));
  Std.Console.WriteLn(Std.Str.IndexOf('aba', 'z'));
  Std.Console.WriteLn(Std.Str.Replace('aaa', 'a', 'b'));
  Std.Console.WriteLn(Std.Array.Length(Std.Str.Split('x,y', ',')));
  Std.Console.WriteLn(Std.Str.Join(['x', 'y'], ':'));
  Std.Console.WriteLn(Std.Str.IsNumeric('42'));
  Std.Console.WriteLn(Std.Str.IsNumeric('nope'))
end.",
    );
    assert_eq!(
        out.lines,
        vec![
            "5", "AB", "ab", "x", "true", "true", "true", "Hel", "0", "-1", "bbb", "2", "x:y",
            "true", "false"
        ]
    );
}

#[test]
fn std_substring_out_of_range_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var S: string := Std.Str.Substring('ab', 0, 5)
end.",
    );
    assert!(msg.contains("Substring") || msg.contains("range"), "{msg}");
}

#[test]
fn std_split_empty_delimiter_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var A: array of string := Std.Str.Split('a', '')
end.",
    );
    assert!(msg.contains("delimiter") || msg.contains("empty"), "{msg}");
}

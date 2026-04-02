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

#[test]
fn std_is_numeric_follows_pascal_number_syntax() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Str.IsNumeric('3.0E-4'));
  Std.Console.WriteLn(Std.Str.IsNumeric('5.'));
  Std.Console.WriteLn(Std.Str.IsNumeric('NaN'))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "false"]);
}

#[test]
fn std_is_numeric_accepts_trimmed_signed_underscored_numbers() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Str.IsNumeric('  +1_024  '));
  Std.Console.WriteLn(Std.Str.IsNumeric(' -0.25E+2 '))
end.",
    );
    assert_eq!(out.lines, vec!["true", "true"]);
}

#[test]
fn std_is_numeric_rejects_malformed_exponent_and_underscores() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Str.IsNumeric('1.0e'));
  Std.Console.WriteLn(Std.Str.IsNumeric('1__0'));
  Std.Console.WriteLn(Std.Str.IsNumeric('1e3'))
end.",
    );
    assert_eq!(out.lines, vec!["false", "false", "false"]);
}

// ── String index S[I] ─────────────────────────────────────────────────────────

#[test]
fn string_index_first_char() {
    let out = compile_and_run(
        "\
program StrIdx;
begin
  var S: string := 'Hello';
  var C: char := S[0];
  Std.Console.WriteLn(C)
end.",
    );
    assert_eq!(out.lines, vec!["H"]);
}

#[test]
fn string_index_last_char() {
    let out = compile_and_run(
        "\
program StrIdxLast;
begin
  var S: string := 'Hello';
  var C: char := S[4];
  Std.Console.WriteLn(C)
end.",
    );
    assert_eq!(out.lines, vec!["o"]);
}

#[test]
fn string_index_loop_over_chars() {
    let out = compile_and_run(
        "\
program StrIdxLoop;
uses Std.Str;
begin
  var S: string := 'abc';
  mutable var I: integer := 0;
  while I < Std.Str.Length(S) do
  begin
    Std.Console.WriteLn(S[I]);
    I := I + 1
  end
end.",
    );
    assert_eq!(out.lines, vec!["a", "b", "c"]);
}

#[test]
fn string_index_out_of_bounds_runtime_error() {
    let msg = compile_run_err(
        "\
program StrIdxOob;
begin
  var S: string := 'Hi';
  var C: char := S[5]
end.",
    );
    assert!(
        msg.contains("out of bounds") || msg.contains("index") || msg.contains("String"),
        "{msg}"
    );
}

#[test]
fn string_index_empty_string_runtime_error() {
    let msg = compile_run_err(
        "\
program StrIdxEmpty;
begin
  var S: string := '';
  var C: char := S[0]
end.",
    );
    assert!(
        msg.contains("out of bounds") || msg.contains("index") || msg.contains("String"),
        "{msg}"
    );
}

#[test]
fn string_index_non_integer_is_sema_error() {
    let err = compile_err(
        "\
program StrIdxBadType;
begin
  var S: string := 'hello';
  var C: char := S['x']
end.",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("integer") || msg.contains("String index"),
        "{msg}"
    );
}

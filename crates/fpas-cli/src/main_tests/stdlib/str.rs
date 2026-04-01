use super::super::support;

// ---------------------------------------------------------------------------
// Length
// ---------------------------------------------------------------------------

#[test]
fn length_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Length('hello'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn length_empty_string() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Length(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn length_qualified() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Length('abc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

// ---------------------------------------------------------------------------
// ToUpper / ToLower
// ---------------------------------------------------------------------------

#[test]
fn to_upper() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper('hello'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "HELLO\n");
}

#[test]
fn to_upper_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn to_upper_already_upper() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToUpper('ABC'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ABC\n");
}

#[test]
fn to_lower() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToLower('HELLO'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn to_lower_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(ToLower(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

// ---------------------------------------------------------------------------
// Trim
// ---------------------------------------------------------------------------

#[test]
fn trim_both_sides() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Trim('  hi  '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hi\n");
}

#[test]
fn trim_no_whitespace() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Trim('abc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "abc\n");
}

#[test]
fn trim_all_whitespace() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Trim('   '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

// ---------------------------------------------------------------------------
// Contains
// ---------------------------------------------------------------------------

#[test]
fn contains_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello world', 'world'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn contains_empty_sub() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Contains('hello', ''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// StartsWith / EndsWith
// ---------------------------------------------------------------------------

#[test]
fn starts_with_true() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(StartsWith('hello', 'hel'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn starts_with_false() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(StartsWith('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn ends_with_true() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(EndsWith('hello', 'llo'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn ends_with_false() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(EndsWith('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

// ---------------------------------------------------------------------------
// Substring
// ---------------------------------------------------------------------------

#[test]
fn substring_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Substring('Hello', 0, 3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hel\n");
}

#[test]
fn substring_full_string() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Substring('abc', 0, 3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "abc\n");
}

#[test]
fn substring_out_of_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Substring('hi', 0, 10))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// IndexOf
// ---------------------------------------------------------------------------

#[test]
fn index_of_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IndexOf('abcabc', 'bc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn index_of_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IndexOf('hello', 'xyz'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

// ---------------------------------------------------------------------------
// Replace
// ---------------------------------------------------------------------------

#[test]
fn replace_all_occurrences() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Replace('aaa', 'a', 'b'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "bbb\n");
}

#[test]
fn replace_no_match() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Replace('hello', 'xyz', '!'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

// ---------------------------------------------------------------------------
// Split
// ---------------------------------------------------------------------------

#[test]
fn split_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str, Std.Array;
begin
  var Parts: array of string := Split('a,b,c', ',');
  WriteLn(Std.Array.Length(Parts));
  WriteLn(Parts[0]);
  WriteLn(Parts[1]);
  WriteLn(Parts[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\na\nb\nc\n");
}

#[test]
fn split_delimiter_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str, Std.Array;
begin
  var Parts: array of string := Split('hello', ',');
  WriteLn(Std.Array.Length(Parts));
  WriteLn(Parts[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\nhello\n");
}

#[test]
fn split_empty_delimiter_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  var Parts: array of string := Split('abc', '')
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Join
// ---------------------------------------------------------------------------

#[test]
fn join_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Join(['a', 'b', 'c'], ':'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "a:b:c\n");
}

#[test]
fn join_single_element() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Join(['only'], ','))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "only\n");
}

#[test]
fn join_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  var Empty: array of string := [];
  WriteLn(Join(Empty, ','))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

// ---------------------------------------------------------------------------
// IsNumeric
// ---------------------------------------------------------------------------

#[test]
fn is_numeric_integer() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IsNumeric('42'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_numeric_real() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IsNumeric('3.14'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn is_numeric_invalid() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IsNumeric('nope'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn is_numeric_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IsNumeric(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn is_numeric_negative() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(IsNumeric('-7'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// Repeat
// ---------------------------------------------------------------------------

#[test]
fn repeat_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(RepeatStr('ab', 3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ababab\n");
}

#[test]
fn repeat_zero_count() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(RepeatStr('x', 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn repeat_negative_count() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(RepeatStr('x', -5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn repeat_empty_string() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(RepeatStr('', 5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

// ---------------------------------------------------------------------------
// PadLeft / PadRight / PadCenter
// ---------------------------------------------------------------------------

#[test]
fn pad_left_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('42', 5, '0'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "00042\n");
}

#[test]
fn pad_left_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('Hello', 3, ' '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_right_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('Hi', 6, '.'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hi....\n");
}

#[test]
fn pad_right_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('Hello', 2, '.'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_center_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hi', 6, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "--Hi--\n");
}

#[test]
fn pad_center_odd_spacing() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hi', 7, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // 7 - 2 = 5 remaining, left gets 2, right gets 3
    assert_eq!(stdout, "--Hi---\n");
}

#[test]
fn pad_center_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hello', 3, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_left_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadLeft width must be >= 0"),
        "stderr: {stderr}"
    );
}

#[test]
fn pad_right_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadRight width must be >= 0"),
        "stderr: {stderr}"
    );
}

#[test]
fn pad_center_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadCenter width must be >= 0"),
        "stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// FromChar
// ---------------------------------------------------------------------------

#[test]
fn from_char_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(FromChar('x', 4))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "xxxx\n");
}

#[test]
fn from_char_zero_count() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(FromChar('x', 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

// ---------------------------------------------------------------------------
// CharAt
// ---------------------------------------------------------------------------

#[test]
fn char_at_first() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hello', 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "H\n");
}

#[test]
fn char_at_last() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hello', 4))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "o\n");
}

#[test]
fn char_at_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hi', 5))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn char_at_negative_index() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hi', -1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// SetCharAt
// ---------------------------------------------------------------------------

#[test]
fn set_char_at_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(SetCharAt('Hello', 0, 'J'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Jello\n");
}

#[test]
fn set_char_at_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(SetCharAt('Hi', 10, 'X'))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Ord / Chr
// ---------------------------------------------------------------------------

#[test]
fn ord_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Ord('A'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "65\n");
}

#[test]
fn ord_zero_char() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Ord('0'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "48\n");
}

#[test]
fn chr_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(65))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "A\n");
}

#[test]
fn chr_invalid_codepoint() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(-1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn chr_rejects_oversized_codepoint() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(4294967296))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("Chr: 4294967296 is not a valid Unicode code point"),
        "stderr: {stderr}"
    );
}

#[test]
fn ord_chr_roundtrip() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(Ord('Z')))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Z\n");
}

// ---------------------------------------------------------------------------
// Insert
// ---------------------------------------------------------------------------

#[test]
fn insert_middle() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hllo', 1, 'e'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn insert_at_start() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('world', 0, 'Hello '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello world\n");
}

#[test]
fn insert_at_end() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hello', 5, '!'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello!\n");
}

#[test]
fn insert_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hi', 10, 'x'))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[test]
fn delete_middle() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hello', 1, 3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Ho\n");
}

#[test]
fn delete_from_start() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hello', 0, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "llo\n");
}

#[test]
fn delete_all() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hi', 0, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn delete_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hi', 0, 10))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Reverse (Std.Str)
// ---------------------------------------------------------------------------

#[test]
fn str_reverse_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Reverse('abc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "cba\n");
}

#[test]
fn str_reverse_empty() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Reverse(''))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn str_reverse_single_char() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Reverse('X'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "X\n");
}

// ---------------------------------------------------------------------------
// TrimLeft / TrimRight
// ---------------------------------------------------------------------------

#[test]
fn trim_left_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(TrimLeft('  hi  '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hi  \n");
}

#[test]
fn trim_left_no_leading_space() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(TrimLeft('hi'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hi\n");
}

#[test]
fn trim_right_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(TrimRight('  hi  '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "  hi\n");
}

#[test]
fn trim_right_no_trailing_space() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(TrimRight('hi'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hi\n");
}

// ---------------------------------------------------------------------------
// LastIndexOf
// ---------------------------------------------------------------------------

#[test]
fn last_index_of_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('abcabc', 'abc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn last_index_of_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('abc', 'z'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn last_index_of_single_occurrence() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('hello', 'ell'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn last_index_of_empty_string() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('', 'x'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

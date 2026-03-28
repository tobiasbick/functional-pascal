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

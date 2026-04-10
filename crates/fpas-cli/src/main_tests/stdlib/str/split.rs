use super::super::super::support;

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

use super::super::super::support;

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

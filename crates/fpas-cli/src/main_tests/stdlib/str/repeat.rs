use super::super::super::support;

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

use super::super::super::support;

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

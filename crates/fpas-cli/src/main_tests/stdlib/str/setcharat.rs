use super::super::super::support;

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

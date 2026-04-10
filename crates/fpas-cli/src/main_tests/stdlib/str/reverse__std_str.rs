use super::super::super::support;

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

use super::super::super::support;

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

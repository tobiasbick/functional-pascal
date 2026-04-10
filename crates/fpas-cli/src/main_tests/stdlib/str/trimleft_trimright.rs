use super::super::super::support;

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

use super::super::super::support;

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

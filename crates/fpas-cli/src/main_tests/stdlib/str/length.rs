use super::super::super::support;

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

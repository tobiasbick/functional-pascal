use super::super::super::support;

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[test]
fn delete_middle() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hello', 1, 3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Ho\n");
}

#[test]
fn delete_from_start() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hello', 0, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "llo\n");
}

#[test]
fn delete_all() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hi', 0, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn delete_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Delete('Hi', 0, 10))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

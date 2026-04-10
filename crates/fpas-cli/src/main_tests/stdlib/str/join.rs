use super::super::super::support;

// ---------------------------------------------------------------------------
// Join
// ---------------------------------------------------------------------------

#[test]
fn join_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Join(['a', 'b', 'c'], ':'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "a:b:c\n");
}

#[test]
fn join_single_element() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Join(['only'], ','))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "only\n");
}

#[test]
fn join_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  var Empty: array of string := [];
  WriteLn(Join(Empty, ','))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

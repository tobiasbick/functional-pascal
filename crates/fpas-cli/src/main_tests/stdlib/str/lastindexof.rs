use super::super::super::support;

// ---------------------------------------------------------------------------
// LastIndexOf
// ---------------------------------------------------------------------------

#[test]
fn last_index_of_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('abcabc', 'abc'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn last_index_of_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('abc', 'z'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn last_index_of_single_occurrence() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('hello', 'ell'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn last_index_of_empty_string() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(LastIndexOf('', 'x'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

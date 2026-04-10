use super::super::super::support;

// ---------------------------------------------------------------------------
// Replace
// ---------------------------------------------------------------------------

#[test]
fn replace_all_occurrences() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Replace('aaa', 'a', 'b'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "bbb\n");
}

#[test]
fn replace_no_match() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Replace('hello', 'xyz', '!'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

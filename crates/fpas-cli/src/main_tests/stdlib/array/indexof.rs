use super::super::super::support;

// ---------------------------------------------------------------------------
// IndexOf
// ---------------------------------------------------------------------------

#[test]
fn index_of_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  WriteLn(IndexOf([10, 20, 30], 20))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn index_of_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  WriteLn(IndexOf([10, 20, 30], 99))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

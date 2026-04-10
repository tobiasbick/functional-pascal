use super::super::super::support;

// ---------------------------------------------------------------------------
// ForEach
// ---------------------------------------------------------------------------

#[test]
fn for_each_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
procedure PrintValue(X: integer);
begin
  WriteLn(X)
end;
begin
  ForEach([10, 20, 30], PrintValue)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "10\n20\n30\n");
}

#[test]
fn for_each_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
procedure PrintValue(X: integer);
begin
  WriteLn(X)
end;
begin
  ForEach([], PrintValue);
  WriteLn('done')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "done\n");
}

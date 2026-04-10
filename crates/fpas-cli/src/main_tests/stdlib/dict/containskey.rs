use super::super::super::support;

// ---------------------------------------------------------------------------
// ContainsKey
// ---------------------------------------------------------------------------

#[test]
fn contains_key_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(Std.Dict.ContainsKey(D, 'Alice'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_key_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(Std.Dict.ContainsKey(D, 'Bob'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

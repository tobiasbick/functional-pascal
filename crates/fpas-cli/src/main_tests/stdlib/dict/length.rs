use super::super::super::support;

// ---------------------------------------------------------------------------
// Length
// ---------------------------------------------------------------------------

#[test]
fn length_normal() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2];
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn length_empty() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := [:];
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

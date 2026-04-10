use super::super::super::support;

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[test]
fn dict_get_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  var R: Option of integer := Std.Dict.Get(D, 'Alice');
  WriteLn(IsSome(R));
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n30\n");
}

#[test]
fn dict_get_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := ['Alice': 30];
  var R: Option of integer := Std.Dict.Get(D, 'Eve');
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn dict_get_empty_dict() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := [:];
  var R: Option of integer := Std.Dict.Get(D, 'X');
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Any
// ---------------------------------------------------------------------------

#[test]
fn any_some_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsNegative(X: integer): boolean;
begin
  return X < 0
end;
begin
  var A: array of integer := [1, -2, 3];
  WriteLn(Any(A, IsNegative))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn any_no_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsNegative(X: integer): boolean;
begin
  return X < 0
end;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Any(A, IsNegative))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn any_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  WriteLn(Any(A, AlwaysTrue))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

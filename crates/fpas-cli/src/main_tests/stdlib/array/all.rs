use super::super::super::support;

// ---------------------------------------------------------------------------
// All
// ---------------------------------------------------------------------------

#[test]
fn all_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsPositive(X: integer): boolean;
begin
  return X > 0
end;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(All(A, IsPositive))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn all_some_fail() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsPositive(X: integer): boolean;
begin
  return X > 0
end;
begin
  var A: array of integer := [1, -2, 3];
  WriteLn(All(A, IsPositive))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn all_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysFalse(X: integer): boolean;
begin
  return false
end;
begin
  var A: array of integer := [];
  WriteLn(All(A, AlwaysFalse))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // All on empty array is vacuously true
    assert_eq!(stdout, "true\n");
}

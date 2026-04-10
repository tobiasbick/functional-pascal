use super::super::super::support;

// ---------------------------------------------------------------------------
// Find
// ---------------------------------------------------------------------------

#[test]
fn find_found() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function IsGreaterThanThree(X: integer): boolean;
begin
  return X > 3
end;
begin
  var A: array of integer := [1, 2, 3, 4, 5];
  var R: Option of integer := Find(A, IsGreaterThanThree);
  WriteLn(IsSome(R));
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n4\n");
}

#[test]
fn find_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function IsGreaterThanTen(X: integer): boolean;
begin
  return X > 10
end;
begin
  var A: array of integer := [1, 2, 3];
  var R: Option of integer := Find(A, IsGreaterThanTen);
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn find_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  var R: Option of integer := Find(A, AlwaysTrue);
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

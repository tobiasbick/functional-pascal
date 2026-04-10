use super::super::super::support;

// ---------------------------------------------------------------------------
// FindIndex
// ---------------------------------------------------------------------------

#[test]
fn find_index_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsGreaterThanFifteen(X: integer): boolean;
begin
  return X > 15
end;
begin
  var A: array of integer := [10, 20, 30];
  var Idx: integer := FindIndex(A, IsGreaterThanFifteen);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn find_index_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsGreaterThanHundred(X: integer): boolean;
begin
  return X > 100
end;
begin
  var A: array of integer := [1, 2, 3];
  var Idx: integer := FindIndex(A, IsGreaterThanHundred);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn find_index_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  var Idx: integer := FindIndex(A, AlwaysTrue);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

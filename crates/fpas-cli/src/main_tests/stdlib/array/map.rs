use super::super::super::support;

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

#[test]
fn map_double() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function Double(X: integer): integer;
begin
  return X * 2
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Doubled: array of integer := Map(Nums, Double);
  WriteLn(Doubled[0]);
  WriteLn(Doubled[1]);
  WriteLn(Doubled[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n4\n6\n");
}

#[test]
fn map_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AddOne(X: integer): integer;
begin
  return X + 1
end;
begin
  var A: array of integer := [];
  var B: array of integer := Map(A, AddOne);
  WriteLn(Length(B))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

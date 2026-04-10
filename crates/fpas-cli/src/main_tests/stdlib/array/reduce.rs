use super::super::super::support;

// ---------------------------------------------------------------------------
// Reduce
// ---------------------------------------------------------------------------

#[test]
fn reduce_sum() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AddAcc(Acc: integer; V: integer): integer;
begin
  return Acc + V
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Sum: integer := Reduce(Nums, 0, AddAcc);
  WriteLn(Sum)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "15\n");
}

#[test]
fn reduce_empty_returns_init() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AddAcc(Acc: integer; V: integer): integer;
begin
  return Acc + V
end;
begin
  var A: array of integer := [];
  var Val: integer := Reduce(A, 99, AddAcc);
  WriteLn(Val)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "99\n");
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[test]
fn filter_evens() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Evens: array of integer := Filter(Nums, IsEven);
  WriteLn(Length(Evens));
  WriteLn(Evens[0]);
  WriteLn(Evens[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n2\n4\n");
}

#[test]
fn filter_none_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;
begin
  var Nums: array of integer := [1, 3, 5];
  var Evens: array of integer := Filter(Nums, IsEven);
  WriteLn(Length(Evens))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

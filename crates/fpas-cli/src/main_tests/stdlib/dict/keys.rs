use super::super::super::support;

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

#[test]
fn keys_preserves_insertion_order() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Array;
begin
  var D: dict of string to integer := ['B': 2, 'A': 1];
  var K: array of string := Std.Dict.Keys(D);
  WriteLn(K[0]);
  WriteLn(K[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "B\nA\n");
}

#[test]
fn keys_empty_dict() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Array;
begin
  var D: dict of string to integer := [:];
  var K: array of string := Std.Dict.Keys(D);
  WriteLn(Std.Array.Length(K))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

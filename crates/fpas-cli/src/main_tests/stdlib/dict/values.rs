use super::super::super::support;

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

#[test]
fn values_preserves_insertion_order() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 10, 'B': 20];
  var V: array of integer := Std.Dict.Values(D);
  WriteLn(V[0]);
  WriteLn(V[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "10\n20\n");
}

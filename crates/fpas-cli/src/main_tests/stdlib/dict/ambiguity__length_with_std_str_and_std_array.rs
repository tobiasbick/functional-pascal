use super::super::super::support;

// ---------------------------------------------------------------------------
// Ambiguity: Length with Std.Str and Std.Array
// ---------------------------------------------------------------------------

#[test]
fn dict_length_ambiguity_requires_qualified() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Array;
begin
  var D: dict of string to integer := ['X': 1];
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

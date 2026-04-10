use super::super::super::support;

// ---------------------------------------------------------------------------
// Dict indexing
// ---------------------------------------------------------------------------

#[test]
fn dict_index_read() {
    let source = r#"program T;
uses Std.Console;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(D['Alice'])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "30\n");
}

#[test]
fn dict_index_missing_key_is_runtime_error() {
    let source = r#"program T;
uses Std.Console;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(D['Bob'])
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn dict_mutable_insert_and_update() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  mutable var D: dict of string to integer := ['A': 1];
  D['A'] := 2;
  D['B'] := 3;
  WriteLn(D['A']);
  WriteLn(D['B']);
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n3\n2\n");
}

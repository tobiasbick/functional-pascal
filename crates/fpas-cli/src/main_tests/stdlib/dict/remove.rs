use super::super::super::support;

// ---------------------------------------------------------------------------
// Remove
// ---------------------------------------------------------------------------

#[test]
fn remove_existing_key() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
  var D2: dict of string to integer := Std.Dict.Remove(D, 'B');
  WriteLn(Std.Dict.Length(D2));
  WriteLn(Std.Dict.ContainsKey(D2, 'B'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\nfalse\n");
}

#[test]
fn remove_nonexistent_key() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1];
  var D2: dict of string to integer := Std.Dict.Remove(D, 'Z');
  WriteLn(Std.Dict.Length(D2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn remove_does_not_mutate_original() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2];
  var D2: dict of string to integer := Std.Dict.Remove(D, 'A');
  WriteLn(Std.Dict.Length(D));
  WriteLn(Std.Dict.Length(D2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n1\n");
}

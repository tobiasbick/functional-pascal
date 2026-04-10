use super::super::super::support;

// ---------------------------------------------------------------------------
// CharAt
// ---------------------------------------------------------------------------

#[test]
fn char_at_first() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hello', 0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "H\n");
}

#[test]
fn char_at_last() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hello', 4))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "o\n");
}

#[test]
fn char_at_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hi', 5))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn char_at_negative_index() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(CharAt('Hi', -1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

use super::super::super::support;

// ---------------------------------------------------------------------------
// Insert
// ---------------------------------------------------------------------------

#[test]
fn insert_middle() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hllo', 1, 'e'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn insert_at_start() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('world', 0, 'Hello '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello world\n");
}

#[test]
fn insert_at_end() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hello', 5, '!'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello!\n");
}

#[test]
fn insert_out_of_bounds() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Std.Str.Insert('Hi', 10, 'x'))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

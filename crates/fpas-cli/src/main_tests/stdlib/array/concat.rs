use super::super::super::support;

// ---------------------------------------------------------------------------
// Concat
// ---------------------------------------------------------------------------

#[test]
fn concat_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var C: array of integer := Concat([1, 2], [3, 4]);
  WriteLn(Length(C));
  WriteLn(C[0]);
  WriteLn(C[3])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "4\n1\n4\n");
}

#[test]
fn concat_empty_left() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var C: array of integer := Concat([], [1, 2]);
  WriteLn(Length(C))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn concat_empty_right() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var C: array of integer := Concat([1, 2], []);
  WriteLn(Length(C))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn concat_both_empty() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var C: array of integer := Concat([], []);
  WriteLn(Length(C))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn concat_rejects_incompatible_element_types() {
    let source = r#"program T;
uses Std.Array;
begin
  var C: array of integer := Concat([1], [true])
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("right array element"), "stderr: {stderr}");
}

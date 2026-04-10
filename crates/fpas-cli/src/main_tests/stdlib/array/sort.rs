use super::super::super::support;

// ---------------------------------------------------------------------------
// Sort
// ---------------------------------------------------------------------------

#[test]
fn sort_integers() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [3, 1, 2];
  var B: array of integer := Sort(A);
  WriteLn(B[0]);
  WriteLn(B[1]);
  WriteLn(B[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n2\n3\n");
}

#[test]
fn sort_already_sorted() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  var B: array of integer := Sort(A);
  WriteLn(B[0]);
  WriteLn(B[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n3\n");
}

#[test]
fn sort_single_element() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [42];
  var B: array of integer := Sort(A);
  WriteLn(B[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn sort_does_not_mutate_original() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [3, 1, 2];
  var B: array of integer := Sort(A);
  WriteLn(A[0]);
  WriteLn(B[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n1\n");
}

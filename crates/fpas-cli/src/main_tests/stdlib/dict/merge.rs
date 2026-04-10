use super::super::super::support;

// ---------------------------------------------------------------------------
// Merge
// ---------------------------------------------------------------------------

#[test]
fn dict_merge_normal() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var A: dict of string to integer := ['A': 1, 'B': 2];
  var B: dict of string to integer := ['C': 3];
  var M: dict of string to integer := Std.Dict.Merge(A, B);
  WriteLn(Std.Dict.Length(M));
  WriteLn(M['A']);
  WriteLn(M['C'])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n1\n3\n");
}

#[test]
fn dict_merge_overlapping_keys() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var A: dict of string to integer := ['A': 1, 'B': 2];
  var B: dict of string to integer := ['B': 9, 'C': 3];
  var M: dict of string to integer := Std.Dict.Merge(A, B);
  WriteLn(M['B'])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // D2 wins on conflict
    assert_eq!(stdout, "9\n");
}

#[test]
fn dict_merge_empty_left() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var A: dict of string to integer := [:];
  var B: dict of string to integer := ['X': 1];
  var M: dict of string to integer := Std.Dict.Merge(A, B);
  WriteLn(Std.Dict.Length(M));
  WriteLn(M['X'])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n1\n");
}

#[test]
fn dict_merge_empty_right() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var A: dict of string to integer := ['X': 1];
  var B: dict of string to integer := [:];
  var M: dict of string to integer := Std.Dict.Merge(A, B);
  WriteLn(Std.Dict.Length(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn dict_merge_both_empty() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var A: dict of string to integer := [:];
  var B: dict of string to integer := [:];
  var M: dict of string to integer := Std.Dict.Merge(A, B);
  WriteLn(Std.Dict.Length(M))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

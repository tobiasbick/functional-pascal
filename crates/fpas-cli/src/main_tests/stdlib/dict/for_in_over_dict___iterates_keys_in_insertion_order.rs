use super::super::super::support;

// ---------------------------------------------------------------------------
// for-in over dict — iterates keys in insertion order
// ---------------------------------------------------------------------------

#[test]
fn for_in_dict_basic() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  for K: string in D do
    WriteLn(K)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Alice\nBob\n");
}

#[test]
fn for_in_dict_insertion_order_preserved() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['C': 3, 'A': 1, 'B': 2];
  for K: string in D do
    WriteLn(K)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "C\nA\nB\n");
}

#[test]
fn for_in_dict_empty_no_iterations() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := [:];
  mutable var Count: integer := 0;
  for K: string in D do
    Count := Count + 1;
  WriteLn(Count)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn for_in_dict_key_used_to_access_value() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Conv;
begin
  var D: dict of string to integer := ['x': 10, 'y': 20];
  for K: string in D do
    WriteLn(K + '=' + IntToStr(D[K]))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "x=10\ny=20\n");
}

#[test]
fn for_in_dict_single_entry() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to boolean := ['ok': true];
  for K: string in D do
    WriteLn(K)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ok\n");
}

#[test]
fn for_in_dict_integer_keys() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Conv;
begin
  var D: dict of integer to string := [1: 'one', 2: 'two', 3: 'three'];
  for K: integer in D do
    WriteLn(IntToStr(K))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n2\n3\n");
}

#[test]
fn for_in_dict_break_stops_iteration() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
  for K: string in D do
  begin
    if K = 'B' then break;
    WriteLn(K)
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "A\n");
}

#[test]
fn for_in_dict_continue_skips_entry() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
  for K: string in D do
  begin
    if K = 'B' then continue;
    WriteLn(K)
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "A\nC\n");
}

#[test]
fn for_in_dict_nested_loops() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Conv;
begin
  var Outer: dict of string to integer := ['a': 1, 'b': 2];
  var Inner: dict of string to integer := ['x': 10, 'y': 20];
  for Ko: string in Outer do
    for Ki: string in Inner do
      WriteLn(Ko + Ki)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ax\nay\nbx\nby\n");
}

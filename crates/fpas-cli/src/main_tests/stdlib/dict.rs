use super::super::support;

// ---------------------------------------------------------------------------
// Length
// ---------------------------------------------------------------------------

#[test]
fn length_normal() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2];
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn length_empty() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := [:];
  WriteLn(Std.Dict.Length(D))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// ContainsKey
// ---------------------------------------------------------------------------

#[test]
fn contains_key_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(Std.Dict.ContainsKey(D, 'Alice'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_key_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict;
begin
  var D: dict of string to integer := ['Alice': 30];
  WriteLn(Std.Dict.ContainsKey(D, 'Bob'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

#[test]
fn keys_preserves_insertion_order() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Array;
begin
  var D: dict of string to integer := ['B': 2, 'A': 1];
  var K: array of string := Std.Dict.Keys(D);
  WriteLn(K[0]);
  WriteLn(K[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "B\nA\n");
}

#[test]
fn keys_empty_dict() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Array;
begin
  var D: dict of string to integer := [:];
  var K: array of string := Std.Dict.Keys(D);
  WriteLn(Std.Array.Length(K))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

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

// ---------------------------------------------------------------------------
// Get
// ---------------------------------------------------------------------------

#[test]
fn dict_get_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  var R: Option of integer := Std.Dict.Get(D, 'Alice');
  WriteLn(IsSome(R));
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n30\n");
}

#[test]
fn dict_get_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := ['Alice': 30];
  var R: Option of integer := Std.Dict.Get(D, 'Eve');
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn dict_get_empty_dict() {
    let source = r#"program T;
uses Std.Console, Std.Dict, Std.Option;
begin
  var D: dict of string to integer := [:];
  var R: Option of integer := Std.Dict.Get(D, 'X');
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

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

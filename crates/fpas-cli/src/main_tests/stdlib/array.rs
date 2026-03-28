use super::super::support;

// ---------------------------------------------------------------------------
// Length
// ---------------------------------------------------------------------------

#[test]
fn length_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn length_empty() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [];
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

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

// ---------------------------------------------------------------------------
// Reverse
// ---------------------------------------------------------------------------

#[test]
fn reverse_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  var R: array of integer := Reverse(A);
  WriteLn(R[0]);
  WriteLn(R[1]);
  WriteLn(R[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n2\n1\n");
}

#[test]
fn reverse_single_element() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [42];
  var R: array of integer := Reverse(A);
  WriteLn(R[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

// ---------------------------------------------------------------------------
// Contains
// ---------------------------------------------------------------------------

#[test]
fn contains_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Contains(A, 2))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn contains_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Contains(A, 99))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

// ---------------------------------------------------------------------------
// IndexOf
// ---------------------------------------------------------------------------

#[test]
fn index_of_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  WriteLn(IndexOf([10, 20, 30], 20))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn index_of_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  WriteLn(IndexOf([10, 20, 30], 99))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

// ---------------------------------------------------------------------------
// Slice
// ---------------------------------------------------------------------------

#[test]
fn slice_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [10, 20, 30, 40];
  var C: array of integer := Slice(A, 1, 2);
  WriteLn(Length(C));
  WriteLn(C[0]);
  WriteLn(C[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n20\n30\n");
}

#[test]
fn slice_out_of_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2];
  var C: array of integer := Slice(A, 0, 10)
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------

#[test]
fn push_appends() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [1, 2];
  Push(A, 3);
  WriteLn(Length(A));
  WriteLn(A[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n3\n");
}

#[test]
fn push_to_empty() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [];
  Push(A, 42);
  WriteLn(Length(A));
  WriteLn(A[0])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n42\n");
}

// ---------------------------------------------------------------------------
// Pop
// ---------------------------------------------------------------------------

#[test]
fn pop_returns_last() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [1, 2, 3];
  var Last: integer := Pop(A);
  WriteLn(Last);
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n2\n");
}

#[test]
fn pop_empty_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  mutable var A: array of integer := [];
  var X: integer := Pop(A)
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

#[test]
fn map_double() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3];
  var Doubled: array of integer := Map(Nums,
    function(X: integer): integer begin return X * 2 end);
  WriteLn(Doubled[0]);
  WriteLn(Doubled[1]);
  WriteLn(Doubled[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n4\n6\n");
}

#[test]
fn map_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [];
  var B: array of integer := Map(A,
    function(X: integer): integer begin return X + 1 end);
  WriteLn(Length(B))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[test]
fn filter_evens() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Evens: array of integer := Filter(Nums,
    function(X: integer): boolean begin return X mod 2 = 0 end);
  WriteLn(Length(Evens));
  WriteLn(Evens[0]);
  WriteLn(Evens[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n2\n4\n");
}

#[test]
fn filter_none_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 3, 5];
  var Evens: array of integer := Filter(Nums,
    function(X: integer): boolean begin return X mod 2 = 0 end);
  WriteLn(Length(Evens))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// Reduce
// ---------------------------------------------------------------------------

#[test]
fn reduce_sum() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Sum: integer := Reduce(Nums, 0,
    function(Acc: integer; V: integer): integer begin return Acc + V end);
  WriteLn(Sum)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "15\n");
}

#[test]
fn reduce_empty_returns_init() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [];
  var Val: integer := Reduce(A, 99,
    function(Acc: integer; V: integer): integer begin return Acc + V end);
  WriteLn(Val)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "99\n");
}

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_length() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Std.Array.Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

// ---------------------------------------------------------------------------
// Edge: array index out of bounds
// ---------------------------------------------------------------------------

#[test]
fn array_index_out_of_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2];
  WriteLn(A[5])
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

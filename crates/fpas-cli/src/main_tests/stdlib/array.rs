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
function Double(X: integer): integer;
begin
  return X * 2
end;
begin
  var Nums: array of integer := [1, 2, 3];
  var Doubled: array of integer := Map(Nums, Double);
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
function AddOne(X: integer): integer;
begin
  return X + 1
end;
begin
  var A: array of integer := [];
  var B: array of integer := Map(A, AddOne);
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
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Evens: array of integer := Filter(Nums, IsEven);
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
function IsEven(X: integer): boolean;
begin
  return X mod 2 = 0
end;
begin
  var Nums: array of integer := [1, 3, 5];
  var Evens: array of integer := Filter(Nums, IsEven);
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
function AddAcc(Acc: integer; V: integer): integer;
begin
  return Acc + V
end;
begin
  var Nums: array of integer := [1, 2, 3, 4, 5];
  var Sum: integer := Reduce(Nums, 0, AddAcc);
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
function AddAcc(Acc: integer; V: integer): integer;
begin
  return Acc + V
end;
begin
  var A: array of integer := [];
  var Val: integer := Reduce(A, 99, AddAcc);
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

// ---------------------------------------------------------------------------
// Find
// ---------------------------------------------------------------------------

#[test]
fn find_found() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function IsGreaterThanThree(X: integer): boolean;
begin
  return X > 3
end;
begin
  var A: array of integer := [1, 2, 3, 4, 5];
  var R: Option of integer := Find(A, IsGreaterThanThree);
  WriteLn(IsSome(R));
  WriteLn(Unwrap(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n4\n");
}

#[test]
fn find_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function IsGreaterThanTen(X: integer): boolean;
begin
  return X > 10
end;
begin
  var A: array of integer := [1, 2, 3];
  var R: Option of integer := Find(A, IsGreaterThanTen);
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn find_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array, Std.Option;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  var R: Option of integer := Find(A, AlwaysTrue);
  WriteLn(IsNone(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

// ---------------------------------------------------------------------------
// FindIndex
// ---------------------------------------------------------------------------

#[test]
fn find_index_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsGreaterThanFifteen(X: integer): boolean;
begin
  return X > 15
end;
begin
  var A: array of integer := [10, 20, 30];
  var Idx: integer := FindIndex(A, IsGreaterThanFifteen);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn find_index_not_found() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsGreaterThanHundred(X: integer): boolean;
begin
  return X > 100
end;
begin
  var A: array of integer := [1, 2, 3];
  var Idx: integer := FindIndex(A, IsGreaterThanHundred);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn find_index_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  var Idx: integer := FindIndex(A, AlwaysTrue);
  WriteLn(Idx)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

// ---------------------------------------------------------------------------
// Any
// ---------------------------------------------------------------------------

#[test]
fn any_some_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsNegative(X: integer): boolean;
begin
  return X < 0
end;
begin
  var A: array of integer := [1, -2, 3];
  WriteLn(Any(A, IsNegative))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn any_no_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsNegative(X: integer): boolean;
begin
  return X < 0
end;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(Any(A, IsNegative))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn any_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysTrue(X: integer): boolean;
begin
  return true
end;
begin
  var A: array of integer := [];
  WriteLn(Any(A, AlwaysTrue))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

// ---------------------------------------------------------------------------
// All
// ---------------------------------------------------------------------------

#[test]
fn all_match() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsPositive(X: integer): boolean;
begin
  return X > 0
end;
begin
  var A: array of integer := [1, 2, 3];
  WriteLn(All(A, IsPositive))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\n");
}

#[test]
fn all_some_fail() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function IsPositive(X: integer): boolean;
begin
  return X > 0
end;
begin
  var A: array of integer := [1, -2, 3];
  WriteLn(All(A, IsPositive))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "false\n");
}

#[test]
fn all_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function AlwaysFalse(X: integer): boolean;
begin
  return false
end;
begin
  var A: array of integer := [];
  WriteLn(All(A, AlwaysFalse))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // All on empty array is vacuously true
    assert_eq!(stdout, "true\n");
}

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

// ---------------------------------------------------------------------------
// FlatMap
// ---------------------------------------------------------------------------

#[test]
fn flat_map_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function Expand(X: integer): array of integer;
begin
  return [X, X * 10]
end;
begin
  var R: array of integer := FlatMap([1, 2, 3], Expand);
  WriteLn(Length(R));
  WriteLn(R[0]);
  WriteLn(R[1]);
  WriteLn(R[4]);
  WriteLn(R[5])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "6\n1\n10\n3\n30\n");
}

#[test]
fn flat_map_empty_results() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function EmptyArray(X: integer): array of integer;
begin
  return []
end;
begin
  var R: array of integer := FlatMap([1, 2, 3], EmptyArray);
  WriteLn(Length(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn flat_map_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function Wrap(X: integer): array of integer;
begin
  return [X]
end;
begin
  var R: array of integer := FlatMap([], Wrap);
  WriteLn(Length(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn flat_map_rejects_scalar_mapper_result() {
    let source = r#"program T;
uses Std.Array;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var X: integer := FlatMap([1, 2], Identity)
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("mapper must return an array"),
        "stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Fill
// ---------------------------------------------------------------------------

#[test]
fn fill_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := Fill(7, 3);
  WriteLn(Length(A));
  WriteLn(A[0]);
  WriteLn(A[2])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n7\n7\n");
}

#[test]
fn fill_zero_count() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := Fill(7, 0);
  WriteLn(Length(A))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn fill_string_elements() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of string := Fill('x', 2);
  WriteLn(A[0]);
  WriteLn(A[1])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "x\nx\n");
}

// ---------------------------------------------------------------------------
// ForEach
// ---------------------------------------------------------------------------

#[test]
fn for_each_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
procedure PrintValue(X: integer);
begin
  WriteLn(X)
end;
begin
  ForEach([10, 20, 30], PrintValue)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "10\n20\n30\n");
}

#[test]
fn for_each_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
procedure PrintValue(X: integer);
begin
  WriteLn(X)
end;
begin
  ForEach([], PrintValue);
  WriteLn('done')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "done\n");
}
